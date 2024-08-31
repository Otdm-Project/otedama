mod connect;

#[tokio::main]
async fn main() {
    println!("Starting WebSocket server...");
    connect::start_websocket_server().await;
}
