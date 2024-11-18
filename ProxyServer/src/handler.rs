use warp::ws::{Message, WebSocket};
use futures_util::{StreamExt, SinkExt};
use crate::subdomain;
use crate::db;

pub async fn handle_socket(ws: WebSocket) {
    let (mut tx, mut rx) = ws.split();
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    let text = msg.to_str().unwrap();
                    println!("ProxyServer received a message: {}", text);

                    let customer_id: usize = text.parse().unwrap_or(0);
                    if customer_id > 0 {
                        println!("Receive instructions from APIServer");
                        // サブドメインを生成して登録
                        match subdomain::generate_and_add_subdomain("192.168.1.100") {
                            Ok(subdomain) => {
                                // サブドメインをDBに保存
                                match db::insert_subdomain_to_db(customer_id, &subdomain) {
                                    Ok(_) => {
                                        println!("Successfully inserted subdomain into DB: {}", subdomain);
                                        // 仮想IPアドレスを取得
                                        match db::get_virtual_ips(customer_id) {
                                            Ok((client_ip, server_ip)) => {
                                                println!("Retrieved IPs: Client IP: {}, Server IP: {}", client_ip, server_ip);
                                                let response = format!("Subdomain: {}, Client IP: {}, Server IP: {}", subdomain, client_ip, server_ip);
                                                tx.send(Message::text(response)).await.unwrap();
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to retrieve IPs from DB: {}", e);
                                                tx.send(Message::text("Error retrieving IPs from DB")).await.unwrap();
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to insert subdomain into DB: {}", e);
                                        tx.send(Message::text("Error inserting subdomain to DB")).await.unwrap();
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to generate and add subdomain: {}", e);
                                tx.send(Message::text("Error generating and adding subdomain")).await.unwrap();
                            }
                        }
                    }
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
