use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use cassandra_cpp::Session;

pub async fn handle_connection(stream: TcpStream, _session: Arc<Session>) {
    let ws_stream = accept_async(stream).await.expect("Error during the websocket handshake occurred");
    println!("New WebSocket connection: {}", ws_stream.get_ref().peer_addr().unwrap());

    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        let msg = msg.expect("Failed to read message");
        if msg.is_text() || msg.is_binary() {
            println!("Received a message: {}", msg.to_text().unwrap());
            write.send(Message::Text("Pong".into())).await.expect("Failed to write message");
        }
    }
}

pub async fn run_websocket_server(session: Arc<Session>) {
    let addr = "<APIServerのIPアドレス>:9001"; // APIServerのIPアドレス
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    println!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let session_clone = session.clone();
        tokio::spawn(async move {
            handle_connection(stream, session_clone).await;
        });
    }
}
