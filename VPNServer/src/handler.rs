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

                                // ここにWireGuard設定を行うコードを追加できます

                                // DBに情報を保存するなどの処理もここに追加可能
                            }
                            Err(e) => {
                                eprintln!("Failed to retrieve public key: {}", e);
                                tx.send(Message::text("Error retrieving public key")).await.unwrap();
                            }
                        }
                    }

                    tx.send(Message::text("Tunnel creation process completed")).await.unwrap();
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }
}
