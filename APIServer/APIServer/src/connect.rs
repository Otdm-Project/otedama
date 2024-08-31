use warp::Filter;
use warp::ws::{Message, WebSocket};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::net::SocketAddr;
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
use url::Url;

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

                    // VPNServerにトンネル生成の指示を送信
                    send_tunnel_creation_request(id).await;

                    tx.send(Message::text("Public key received and stored, tunnel creation requested")).await.unwrap();
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
        .arg("3.91.133.20")
        .arg("-e")
        .arg(insert_query)
        .output()?;
    Ok(())
}

async fn send_tunnel_creation_request(customer_id: usize) {
    let url = Url::parse("ws://54.89.71.141:8090/ws").unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect to VPNServer");

    let (mut write, mut read) = ws_stream.split();

    let send_task = tokio::spawn(async move {
        let msg = TungsteniteMessage::text(customer_id.to_string());
        write.send(msg).await.expect("Failed to send customer ID");
    });

    let receive_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            if msg.is_text() {
                println!("Received from VPNServer: {}", msg.to_text().unwrap());
            }
        }
    });

    send_task.await.unwrap();
    receive_task.await.unwrap();
}

pub async fn start_websocket_server() {
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handle_socket)
        });

    let addr: SocketAddr = "172.31.84.182:8080".parse().expect("Unable to parse socket address");
    warp::serve(ws_route).run(addr).await;
}
