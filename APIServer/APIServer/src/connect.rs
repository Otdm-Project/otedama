use warp::Filter;
use warp::ws::{Message, WebSocket};
use std::sync::atomic::{AtomicUsize, Ordering};
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
use url::Url;
use std::process::Command;

static ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

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

                        // DBServerから顧客情報を取得して内容を表示し、APIClientに送信
                        let customer_info = retrieve_customer_info_from_db(customer_id);
                        if let Some(info) = customer_info {
                            println!("Retrieved customer info: {:?}", info);

                            let response = format!(
                                "顧客情報:\n顧客公開鍵: {}\nサーバ公開鍵: {}\n顧客IP: {}\nサーバIP: {}\nサブドメイン: {}",
                                info.client_public_key,
                                info.server_public_key,
                                info.vpn_ip_client,
                                info.vpn_ip_server,
                                info.subdomain
                            );

                            println!("Preparing to send customer info to APIClient: {}", response);
                            // 顧客情報メッセージを送信
                            if let Err(e) = tx.send(Message::text(response)).await {
                                eprintln!("Error sending customer info to client: {}", e);
                            } else {
                                println!("Customer info message sent successfully");
                            }

                            // 顧客情報メッセージの後で「Operation completed」を送信
                            println!("Sending 'Operation completed' message");
                            if let Err(e) = tx.send(Message::text("Operation completed")).await {
                                eprintln!("Error sending 'Operation completed': {}", e);
                            } else {
                                println!("'Operation completed' message sent successfully");
                            }
                        } else {
                            println!("Failed to retrieve customer info");
                            if let Err(e) = tx.send(Message::text("顧客情報の取得に失敗しました")).await {
                                eprintln!("Error sending failure message to client: {}", e);
                            }
                        }
                    } else {
                        println!("Received: {}", text);

                        let id = ID_COUNTER.fetch_add(1, Ordering::SeqCst);
                        send_to_db(id, text).expect("Failed to send data to DB");

                        // VPNServerへのトンネル生成の指示
                        send_tunnel_creation_request(id).await;
                    }
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // WebSocket接続を適切に終了
    if let Err(e) = tx.close().await {
        eprintln!("Failed to close WebSocket connection: {}", e);
    }
}

fn send_to_db(id: usize, public_key: &str) -> std::io::Result<()> {
    let insert_query = format!(
        "INSERT INTO customer_data.customer_info (customer_id, client_public_key) VALUES ({}, '{}');",
        id, public_key
    );
    Command::new("cqlsh")
        .arg("10.0.10.40")
        .arg("--cqlversion=3.4.6")
        .arg("-e")
        .arg(insert_query)
        .output()?;
    Ok(())
}

async fn send_tunnel_creation_request(customer_id: usize) {
    let url = Url::parse("ws://10.0.10.20:8090/ws").unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect to VPNServer");
    let (mut write, _) = ws_stream.split();
    let msg = TungsteniteMessage::text(customer_id.to_string());
    write.send(msg).await.expect("Failed to send customer ID");
    println!("Sent tunnel creation request for Customer ID: {}", customer_id);
}

async fn send_subdomain_creation_request(customer_id: usize) {
    let url = Url::parse("ws://10.0.10.30:8100/ws").unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect to ProxyServer");

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
        .arg("--cqlversion=3.4.6")
        .arg("-e")
        .arg(&select_query)
        .output()
        .expect("Failed to execute query");

    println!("Raw output from DB: {}", String::from_utf8_lossy(&output.stdout));

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        parse_customer_info(&output_str)
    } else {
        eprintln!("Failed to retrieve customer info: {}", String::from_utf8_lossy(&output.stderr));
        None
    }
}

fn parse_customer_info(output: &str) -> Option<CustomerInfo> {
    let mut lines = output.lines();

    for _ in 0..3 {
        lines.next();
    }

    for line in lines {
        if line.trim().is_empty() || line.starts_with('-') {
            continue;
        }

        let fields: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        if fields.len() == 5 {
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

#[derive(Debug)]
struct CustomerInfo {
    client_public_key: String,
    server_public_key: String,
    vpn_ip_client: String,
    vpn_ip_server: String,
    subdomain: String,
}

pub async fn start_websocket_server() {
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handle_socket)
        });

    let addr = "0.0.0.0:8080".parse::<std::net::SocketAddr>().expect("Unable to parse socket address");
    warp::serve(ws_route).run(addr).await;
}
