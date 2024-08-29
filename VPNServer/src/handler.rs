use warp::ws::{Message, WebSocket};
use futures_util::{StreamExt, SinkExt};
use crate::db;
use crate::wireguard;

pub async fn handle_socket(ws: WebSocket) {
    let (mut tx, mut rx) = ws.split();
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    let text = msg.to_str().unwrap();
                    println!("Received tunnel creation request for Customer ID: {}", text);

                    let customer_id: usize = text.parse().unwrap_or(0);
                    if customer_id > 0 {
                        match db::get_public_key(customer_id) {
                            Ok(client_public_key) => {
                                println!("Retrieved client public key: {}", client_public_key);

                                // WireGuard鍵ペアの生成
                                let (server_private_key, server_public_key) = wireguard::generate_keypair();
                                println!("Generated WireGuard keys: Private Key: {}, Public Key: {}", server_private_key, server_public_key);

                                // 仮想IPアドレスの割り当て
                                let client_ip = wireguard::allocate_ip_address();
                                let server_ip = "100.64.0.1".to_string();
                                println!("Allocated IP addresses: Server IP: {}, Client IP: {}", server_ip, client_ip);

                                // DBに鍵ペアとIPアドレスを保存
                                match db::insert_tunnel_data(customer_id, &server_public_key, &client_ip, &server_ip) {
                                    Ok(_) => println!("Successfully inserted tunnel data into DB"),
                                    Err(e) => {
                                        eprintln!("Failed to insert tunnel data into DB: {}", e);
                                        tx.send(Message::text("Error saving tunnel data")).await.unwrap();
                                    }
                                }

                                // WireGuardのPeer設定を追加
                                match wireguard::add_peer_to_config(&client_public_key, &client_ip) {
                                    Ok(_) => println!("Successfully added peer to WireGuard config"),
                                    Err(e) => {
                                        eprintln!("Failed to add peer to WireGuard config: {}", e);
                                        tx.send(Message::text("Error adding peer to config")).await.unwrap();
                                    }
                                }

                                // 設定ファイルの内容を表示
                                match wireguard::read_config() {
                                    Ok(config_content) => {
                                        println!("Current WireGuard config:\n{}", config_content);
                                        tx.send(Message::text("Tunnel creation and peer configuration completed")).await.unwrap();
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to read WireGuard config: {}", e);
                                        tx.send(Message::text("Error reading config")).await.unwrap();
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to retrieve public key: {}", e);
                                tx.send(Message::text("Error retrieving public key")).await.unwrap();
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
}
