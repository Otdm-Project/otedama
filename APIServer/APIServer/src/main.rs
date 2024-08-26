mod websocket;

#[tokio::main]
async fn main() {
    println!("Starting WebSocket server...");
    // WebSocketサーバーを起動
    websocket::start_websocket_server().await;
}
