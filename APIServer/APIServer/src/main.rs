use warp::Filter;
use warp::ws::{Message, WebSocket};
use std::sync::atomic::{AtomicUsize, Ordering};
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
use url::Url;
use std::process::Command;
use serde::{Serialize};
use serde_json::json;
use std::io::Result;
use tokio::time::Duration;

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

    // メイン関数が終了しないように待機
    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
}

async fn handle_socket(ws: WebSocket) {
    let (mut tx, mut rx) = ws.split();
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    let text = msg.to_str().unwrap();
                    println!("Received: {}", text);

                    // DBにデータを送信
                    let id = ID_COUNTER.fetch_add(1, Ordering::SeqCst);
                    println!("{},{}", id, text);
                    if let Err(e) = send_to_db(id, text) {
                        eprintln!("Failed to send data to DB: {:?}", e);
                        tx.send(Message::text(json!({"status": "error", "message": "データ送信に失敗しました"}).to_string())).await.unwrap();
                        continue;
                    }

                    println!("DB Insert completed for ID: {}", id);

                    // トンネル生成リクエストを送信し完了を待機
                    send_tunnel_creation_request(id).await;
                    println!("Tunnel creation request completed for ID: {}", id);

                    // サブドメイン生成リクエストを送信し完了を待機
                    send_subdomain_creation_request(id).await;
                    println!("Subdomain creation request completed for ID: {}", id);

                    // DBの更新を確認するためにリトライ
                    if let Some(info) = wait_for_db_update(id).await {
                        let response = serde_json::to_string(&info).expect("Failed to serialize customer info");
                        tx.send(Message::text(response)).await.unwrap();
                        println!("Customer info sent successfully: {:?}", info);
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

async fn wait_for_db_update(customer_id: usize) -> Option<CustomerInfo> {
    let max_retries = 10; // 最大リトライ回数
    let retry_interval = Duration::from_millis(500); // リトライ間隔

    for attempt in 1..=max_retries {
        if let Some(info) = retrieve_customer_info_from_db(customer_id) {
            if info.server_public_key != "null" && info.vpn_ip_client != "null" && info.vpn_ip_server != "null" && info.subdomain != "null" {
                println!("DB Update verified for ID: {}. Retrieved info: {:?}", customer_id, info);
                return Some(info); // 更新済みのデータを確認
            } else {
                println!("DB Update check attempt {} failed for ID: {}", attempt, customer_id);
            }
        }

        // リトライ間隔の待機
        tokio::time::sleep(retry_interval).await;
    }

    println!("DB Update confirmation failed after {} attempts for ID: {}", max_retries, customer_id);
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

async fn start_websocket_server() {
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handle_socket)
        });

    let addr = "0.0.0.0:8080".parse::<std::net::SocketAddr>().expect("Unable to parse socket address");
    warp::serve(ws_route).run(addr).await;
}

