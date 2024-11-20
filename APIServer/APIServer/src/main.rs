use warp::Filter;
use warp::ws::{Message, WebSocket};
use std::sync::atomic::{AtomicUsize, Ordering};
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
use url::Url;
use std::process::Command;
use serde::{Serialize};
use serde_json::{json, to_string};
use std::io::Result;
use tokio::time::{sleep, Duration};

mod monitoring;

// IDに使用する値のカウンタ
static ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[tokio::main]
async fn main() {
    println!("Starting WebSocket server...");

    // WebSocketサーバーを非同期タスクで起動
    tokio::spawn(async {
        start_websocket_server().await;
    });

    // monitoring関数を別スレッドで非同期タスクとして実行
    tokio::task::spawn_blocking(|| {
        println!("Starting monitoring...");
        monitoring();
    })
    .await
    .expect("Failed to run monitoring");

    // メイン関数が終了しないように待機
    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
}

async fn handle_socket(ws: WebSocket) {
    let (mut tx, mut rx) = ws.split();
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    println!("実行！");
                    let text = msg.to_str().unwrap();
                    println!("Received: {}", text);

                    // DBにデータを送信
                    let id = ID_COUNTER.fetch_add(1, Ordering::SeqCst);
                    println!("{},{}", id, text);
                    if let Err(e) = send_to_db(id, text) {
                        eprintln!("Failed to send data to DB: {:?}", e);
                        let error_message = json!({
                            "status": "error",
                            "message": "データ送信に失敗しました"
                        });
                        tx.send(Message::text(error_message.to_string())).await.unwrap();
                        continue;
                    }

                    println!("DB Insert completed for ID: {}", id);

                    // トンネルとサブドメインの生成リクエストを送信し、完了を待機
                    send_tunnel_creation_request(id).await;
                    println!("Tunnel creation request completed for ID: {}", id);

                    send_subdomain_creation_request(id).await;
                    println!("Subdomain creation request completed for ID: {}", id);

                    // 非同期タスクの完了を待機するための遅延処理を追加
                    sleep(Duration::from_secs(2)).await; 

                    // DBから顧客情報を取得して応答
                    if let Some(info) = retrieve_customer_info_from_db(id) {
                        let response = to_string(&info).expect("Failed to serialize customer info");
                        
                        // 顧客情報のメッセージ送信
                        if let Err(e) = tx.send(Message::text(response)).await {
                            eprintln!("Failed to send customer info: {:?}", e);
                        } else {
                            println!("Customer info sent successfully");
                        }
                    } else {
                        let error_message = json!({
                            "status": "error",
                            "message": "顧客情報の取得に失敗しました"
                        });
                        tx.send(Message::text(error_message.to_string())).await.unwrap();
                    }
                    // メッセージが正常に処理されたことを通知
                    let success_message = json!({
                        "status": "success",
                        "message": "Operation completed"
                    });
                    tx.send(Message::text(success_message.to_string())).await.unwrap();
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }

    if let Err(e) = tx.close().await {
        eprintln!("Failed to close WebSocket connection: {}", e);
    }
}

#[derive(Debug, Serialize)]
struct CustomerInfo {
    client_public_key: String,
    server_public_key: String,
    vpn_ip_client: String,
    vpn_ip_server: String,
    subdomain: String,
}

fn parse_customer_info(output: &str) -> Option<CustomerInfo> {
    let mut lines = output.lines();

    while let Some(line) = lines.next() {
        if line.contains("client_public_key") {
            lines.next();
            break;
        }
    }

    if let Some(line) = lines.next() {
        let fields: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        if fields.len() == 5 && fields.iter().all(|f| !f.is_empty()) {
            return Some(CustomerInfo {
                client_public_key: fields[0].to_string(),
                server_public_key: fields[1].to_string(),
                vpn_ip_client: fields[2].to_string(),
                vpn_ip_server: fields[3].to_string(),
                subdomain: fields[4].to_string(),
            });
        }
    }
    None
}

fn send_to_db(id: usize, public_key: &str) -> Result<()> {
    let insert_query = format!(
        "INSERT INTO customer_data.customer_info (customer_id, client_public_key) VALUES ({}, '{}');",
        id, public_key
    );
    Command::new("cqlsh")
        .arg("10.0.10.40")
        .arg("-e")
        .arg(insert_query)
        .output()?;
    Ok(())
}

async fn send_tunnel_creation_request(customer_id: usize) {
    let url = Url::parse("ws://10.0.10.20:8090/ws").unwrap();
    let (ws_stream, _) = connect_async(url.as_str()).await.expect("Failed to connect to VPNServer");
    let (mut write, _) = ws_stream.split();
    let msg = TungsteniteMessage::text(customer_id.to_string());
    write.send(msg).await.expect("Failed to send customer ID");
    println!("Sent tunnel creation request for Customer ID: {}", customer_id);
}

async fn send_subdomain_creation_request(customer_id: usize) {
    let url = Url::parse("ws://10.0.10.30:8100/ws").unwrap();
    let (ws_stream, _) = connect_async(url.as_str()).await.expect("Failed to connect to ProxyServer");
    let (mut write, _) = ws_stream.split();
    let msg = TungsteniteMessage::text(customer_id.to_string());
    write.send(msg).await.expect("Failed to send subdomain creation request");
    println!("Sent subdomain creation request for Customer ID: {}", customer_id);
}

fn retrieve_customer_info_from_db(customer_id: usize) -> Option<CustomerInfo> {
    let select_query = format!(
        "SELECT client_public_key, server_public_key, vpn_ip_client, vpn_ip_server, subdomain FROM customer_data.customer_info WHERE customer_id = {};",
        customer_id
    );

    let output = Command::new("cqlsh")
        .arg("10.0.10.40")
        .arg("-e")
        .arg(&select_query)
        .output()
        .expect("Failed to execute query");

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        println!("Raw query result: {}", output_str); 

        parse_customer_info(&output_str)
    } else {
        eprintln!("Failed to retrieve customer info: {}", String::from_utf8_lossy(&output.stderr));
        None
    }
}

async fn start_websocket_server() {
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handle_socket)
        });

    let addr = "0.0.0.0:8080".parse::<std::net::SocketAddr>().expect("Unable to parse socket address");
    warp::serve(ws_route).run(addr).await;
}

fn monitoring() {
    let client_handle = std::thread::spawn(|| {
        println!("monitoring C start!");
        monitoring::start_client();
        
    });

    client_handle.join().unwrap();
}
