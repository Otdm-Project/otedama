mod connect;

#[tokio::main]
async fn main() {
    println!("Starting WebSocket server...");
    connect::start_websocket_server().await;
}

[ec2-user@ip-172-31-90-93 src]$ cat connect.rs 
use warp::Filter;
use warp::ws::{Message, WebSocket};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::net::SocketAddr;
use futures_util::{StreamExt, SinkExt};

static ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

async fn handle_socket(ws: WebSocket) {
    let (mut tx, mut rx) = ws.split();
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    let text = msg.to_str().unwrap();
                    println!("Received: {}", text);

                    let id = ID_COUNTER.fetch_add(1, Ordering::SeqCst);
                    send_to_db(id, text).expect("Failed to send data to DB");

                    tx.send(Message::text("Public key received and stored")).await.unwrap();
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }
}

fn send_to_db(id: usize, public_key: &str) -> std::io::Result<()> {
    let insert_query = format!(
        "INSERT INTO customer_data.customer_info (customer_id, client_public_key) VALUES ({}, '{}');",
        id, public_key
    );

    Command::new("cqlsh")
        .arg("DBServerのグローバルIPアドレス")
        .arg("-e")
        .arg(insert_query)
        .output()?;
    Ok(())
}

pub async fn start_websocket_server() {
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handle_socket)
        });

    let addr: SocketAddr = "<APIServerのプライベートIPアドレス>:8080".parse().expect("Unable to parse socket address");
    warp::serve(ws_route).run(addr).await;
}
