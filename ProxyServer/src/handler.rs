use warp::ws::{Message, WebSocket};
use futures_util::{StreamExt, SinkExt};
use crate::subdomain;
use crate::db;
use std::fs::OpenOptions;
use std::io::Write;

pub async fn handle_socket(ws: WebSocket) {
    println!("New WebSocket connection established.");

    let (mut tx, mut rx) = ws.split();
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    let text = msg.to_str().unwrap();
                    println!("ProxyServer received a message: {}", text);

                    let customer_id: usize = text.parse().unwrap_or(0);
                    if customer_id > 0 {
                        println!("PROXY指示受信");

                        match subdomain::generate_subdomain() {
                            Ok(subdomain) => {
                                match db::insert_subdomain_to_db(customer_id, &subdomain) {
                                    Ok(_) => {
                                        println!("Successfully inserted subdomain into DB: {}", subdomain);
                                        match db::get_virtual_ips(customer_id) {
                                            Ok((client_ip, server_ip)) => {
                                                println!("Retrieved IPs: Client IP: {}, Server IP: {}", client_ip, server_ip);
                                                if let Err(e) = update_haproxy_config(&client_ip, &server_ip) {
                                                    eprintln!("Failed to update HAProxy config: {}", e);
                                                } else {
                                                    println!("HAProxy config updated successfully.");
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to retrieve IPs from DB: {}", e);
                                            }
                                        }
                                        tx.send(Message::text("Subdomain generation and DB insertion completed")).await.unwrap();
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to insert subdomain into DB: {}", e);
                                        tx.send(Message::text("Error inserting subdomain to DB")).await.unwrap();
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to generate subdomain: {}", e);
                                tx.send(Message::text("Error generating subdomain")).await.unwrap();
                            }
                        }
                    } else {
                        eprintln!("Invalid Customer ID received: {}", text);
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

fn update_haproxy_config(client_ip: &str, server_ip: &str) -> std::io::Result<()> {
    let mut haproxy_cfg = OpenOptions::new()
        .write(true)
        .append(true)
        .open("/etc/haproxy/haproxy.cfg")?;

    writeln!(haproxy_cfg, "    server {} {}:80 check", client_ip, server_ip)?;
    Ok(())
}
