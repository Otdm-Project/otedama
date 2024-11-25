use warp::ws::{Message, WebSocket};
use futures_util::{StreamExt, SinkExt};
use crate::subdomain;
use crate::db;
use tokio::time::{sleep, Duration};

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

                        // IPアドレスを取得
                        if let Some((client_ip, server_ip)) = wait_for_virtual_ips(customer_id).await {
                            println!("Retrieved IPs: Client IP: {}, Server IP: {}", client_ip, server_ip);

                            // サブドメインを生成して登録
                            match subdomain::generate_and_add_subdomain(&client_ip, &server_ip) {
                                Ok(subdomain) => {
                                    // サブドメインをDBに保存
                                    match db::insert_subdomain_to_db(customer_id, &subdomain) {
                                        Ok(_) => {
                                            println!("Successfully inserted subdomain into DB: {}", subdomain);

                                            let response = format!(
                                                "Subdomain: {}, Client IP: {}, Server IP: {}",
                                                subdomain, client_ip, server_ip
                                            );
                                            tx.send(Message::text(response)).await.unwrap();
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
                        } else {
                            eprintln!("Failed to retrieve IPs from DB for Customer ID: {}", customer_id);
                            tx.send(Message::text("Error retrieving IPs from DB")).await.unwrap();
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

async fn wait_for_virtual_ips(customer_id: usize) -> Option<(String, String)> {
    let max_retries = 10; // 最大リトライ回数
    let retry_interval = Duration::from_millis(500); // リトライ間隔

    for attempt in 1..=max_retries {
        if let Ok((client_ip, server_ip)) = db::get_virtual_ips(customer_id) {
            if client_ip != "null" && server_ip != "null" {
                println!("DB Update verified for ID: {}. Retrieved IPs: Client IP: {}, Server IP: {}", customer_id, client_ip, server_ip);
                return Some((client_ip, server_ip));
            } else {
                println!("DB Update check attempt {} failed for ID: {}", attempt, customer_id);
            }
        }

        // リトライ間隔の待機
        sleep(retry_interval).await;
    }

    println!("DB Update confirmation failed after {} attempts for ID: {}", max_retries, customer_id);
    None
}
