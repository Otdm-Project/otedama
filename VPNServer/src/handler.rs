use warp::ws::{Message, WebSocket};
use futures_util::{StreamExt, SinkExt};
use crate::db;
use crate::wireguard;
use tracing::{info, error};

pub async fn handle_socket(ws: WebSocket) {
    let (mut tx, mut rx) = ws.split();
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    let text = msg.to_str().unwrap();
                    info!("Received tunnel creation request for Customer ID: {}", text);
                    let customer_id: usize = text.parse().unwrap_or(0);
                    if customer_id > 0 {
                        match db::get_public_key(customer_id) {
                            Ok(client_public_key) => {
                                info!("Retrieved client public key: {}", client_public_key);

                                // サーバの公開鍵を取得
                                let private_key = std::fs::read_to_string("/etc/wireguard/privatekey")
                                    .expect("Failed to read server private key");
                                let server_public_key = wireguard::get_server_public_key(&private_key);

                                // 仮想IPアドレスの割り当て
                                let client_ip = wireguard::allocate_ip_address();
                                let server_ip = "100.64.0.1".to_string();

                                // DBにトンネルデータを保存
                                match db::insert_tunnel_data(customer_id, &server_public_key, &client_public_key, &client_ip, &server_ip) {
                                    Ok(_) => info!("Successfully inserted tunnel data into DB for Customer ID: {}", customer_id),
                                    Err(e) => {
                                        error!("Failed to insert tunnel data into DB for Customer ID: {}: {}", customer_id, e);
                                        tx.send(Message::text("Error saving tunnel data")).await.unwrap();
                                    }
                                }

                                // WireGuardのPeer設定の追加
                                match wireguard::add_peer_to_wireguard(&client_public_key, &client_ip) {
                                    Ok(_) => info!("Successfully added peer to WireGuard config for Customer ID: {}", customer_id),
                                    Err(e) => {
                                        error!("Failed to add peer to WireGuard config for Customer ID: {}: {}", customer_id, e);
                                        tx.send(Message::text("Error adding peer to config")).await.unwrap();
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to retrieve public key for Customer ID {}: {}", customer_id, e);
                                tx.send(Message::text("Error retrieving public key")).await.unwrap();
                            }
                        }
                    } else {
                        error!("Invalid customer_id received: {}", text);
                        tx.send(Message::text("Invalid Customer ID")).await.unwrap();
                    }
                }
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    if let Err(e) = tx.close().await {
        error!("Failed to close WebSocket connection: {}", e);
    }
}
