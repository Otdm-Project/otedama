use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};

#[tokio::main]
async fn main() {
    // WebSocketサーバがリッスンするアドレスとポートを指定します
    let addr = "<APIServerのIPアドレス>:9001";
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");

    println!("Listening on: {}", addr);

    // 新しい接続を待機します
    while let Ok((stream, _)) = listener.accept().await {
        // 各接続に対して非同期タスクを生成します
        tokio::spawn(handle_connection(stream));
    }
}

async fn handle_connection(stream: TcpStream) {
    // WebSocket接続を確立します
    let ws_stream = accept_async(stream).await.expect("Error during the websocket handshake occurred");
    println!("New WebSocket connection: {}", ws_stream.get_ref().peer_addr().unwrap());

    // 読み取りと書き込みのハーフを分割します
    let (mut write, mut read) = ws_stream.split();

    // メッセージの待機ループ
    while let Some(msg) = read.next().await {
        let msg = msg.expect("Failed to read message");
        if msg.is_text() || msg.is_binary() {
            println!("Received a message: {}", msg.to_text().unwrap());
            // 応答を送信します
            write.send(Message::Text("Pong".into())).await.expect("Failed to write message");
        }
    }
}
