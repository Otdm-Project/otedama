use warp::Filter;
use warp::ws::{Message, WebSocket};
use std::sync::atomic::{AtomicUsize, Ordering};
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
use url::Url;
use std::process::Command;

// IDに使用する値のカウンタ
static ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[tokio::main]
async fn main() {
    println!("Starting WebSocket server...");
    start_websocket_server().await;
}

async fn handle_socket(ws: WebSocket) {
    let (mut tx, mut rx) = ws.split();
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    let text = msg.to_str().unwrap();
                    if text.contains("INSERT completed for Customer ID") {
                        println!("Received notification from VPNServer: {}", text);

                        let customer_id: usize = text.split_whitespace().last().unwrap().parse().unwrap();
                        send_subdomain_creation_request(customer_id).await;

                        // DBから顧客情報を取得し、APIClientにスマートに送信
                        if let Some(info) = retrieve_customer_info_from_db(customer_id) {
                            println!(
                                "取得した顧客情報:\n顧客公開鍵: {}\nサーバ公開鍵: {}\n顧客IP: {}\nサーバIP: {}\nサブドメイン: {}",
                                info.client_public_key,
                                info.server_public_key,
                                info.vpn_ip_client,
                                info.vpn_ip_server,
                                info.subdomain
                            );
                            let response = format!(
                                "顧客情報:\n\
                                顧客公開鍵: {}\n\
                                サーバ公開鍵: {}\n\
                                顧客IP: {}\n\
                                サーバIP: {}\n\
                                サブドメイン: {}",
                                info.client_public_key,
                                info.server_public_key,
                                info.vpn_ip_client,
                                info.vpn_ip_server,
                                info.subdomain
                            );
                            tx.send(Message::text(response)).await.expect("Failed to send customer info");
                        } else {
                            tx.send(Message::text("顧客情報の取得に失敗しました")).await.unwrap();
                        }
                    } else {
                        println!("Received: {}", text);

                        let id = ID_COUNTER.fetch_add(1, Ordering::SeqCst);
                        send_to_db(id, text).expect("Failed to send data to DB");

                        // VPNServerへのトンネル生成の指示
                        send_tunnel_creation_request(id).await;
                    }

                    tx.send(Message::text("Operation completed")).await.unwrap();
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }

    println!("接続が切断されました。");
}

// DBServerに対して生成したID,受領した公開鍵をINSERT
fn send_to_db(id: usize, public_key: &str) -> std::io::Result<()> {
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

// VPNServerにIDとトンネル生成指示
async fn send_tunnel_creation_request(customer_id: usize) {
    let url = Url::parse("ws://10.0.10.20:8090/ws").unwrap();
    let (ws_stream, _) = connect_async(url.as_str()).await.expect("Failed to connect to VPNServer");
    let (mut write, _) = ws_stream.split();
    let msg = TungsteniteMessage::text(customer_id.to_string());
    write.send(msg).await.expect("Failed to send customer ID");
    println!("Sent tunnel creation request for Customer ID: {}", customer_id);
}

// ProxyServerにサブドメイン生成指示
async fn send_subdomain_creation_request(customer_id: usize) {
    let url = Url::parse("ws://10.0.10.30:8100/ws").unwrap();
    let (ws_stream, _) = connect_async(url.as_str()).await.expect("Failed to connect to ProxyServer");
    let (mut write, _) = ws_stream.split();
    let msg = TungsteniteMessage::text(customer_id.to_string());
    write.send(msg).await.expect("Failed to send subdomain creation request");
    println!("Sent subdomain creation request for Customer ID: {}", customer_id);
}

// DBServerから顧客情報を取得
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
        parse_customer_info(&output_str)
    } else {
        eprintln!("Failed to retrieve customer info: {}", String::from_utf8_lossy(&output.stderr));
        None
    }
}

// 顧客情報の解析
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

// 顧客情報構造体
#[derive(Debug)]
struct CustomerInfo {
    client_public_key: String,
    server_public_key: String,
    vpn_ip_client: String,
    vpn_ip_server: String,
    subdomain: String,
}

// WebSocket待受処理
async fn start_websocket_server() {
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handle_socket)
        });

    let addr = "0.0.0.0:8080".parse::<std::net::SocketAddr>().expect("Unable to parse socket address");
    warp::serve(ws_route).run(addr).await;
}
