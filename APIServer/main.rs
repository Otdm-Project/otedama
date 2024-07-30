mod api;
mod websocket;

use tokio::net::TcpListener;
use std::sync::Arc;
use cassandra_cpp::*;

#[tokio::main]
async fn main() {
    let contact_points = "<DBServerのIPアドレス>"; // DBServerのIPアドレス
    let session = api::create_session(contact_points).await.expect("Failed to create session");
    let session = Arc::new(session);

    // HTTP APIとWebSocketサーバの起動
    tokio::spawn(api::run_api_server(session.clone()));
    websocket::run_websocket_server(session.clone()).await;
}
