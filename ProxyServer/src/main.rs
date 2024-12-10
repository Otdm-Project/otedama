mod handler;
mod subdomain; 
mod db; 
mod monitoring;

use warp::Filter;
use std::net::SocketAddr;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // ログ初期化
    tracing_subscriber::fmt()
        .with_target(false)
        .init();

    // WebSocketサーバーを非同期タスクで起動
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handler::handle_socket)
        });

    let addr = "0.0.0.0:8100".parse::<SocketAddr>().expect("Unable to parse socket address");
    info!("ProxyServer is running on {}", addr);

    tokio::spawn(async move {
        warp::serve(ws_route).run(addr).await;
    });

    // monitoring関数を別スレッドで非同期タスクとして実行
    tokio::task::spawn_blocking(|| {
        info!("Starting monitoring...");
        monitoring();
    })
    .await
    .expect("Failed to run monitoring");

    // メイン関数が終了しないように待機
    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
}

// monitoring関数を呼び出す
fn monitoring() {
    let client_handle = std::thread::spawn(|| {
        info!("monitoring C start!");
        monitoring::start_client();
    });

    client_handle.join().unwrap();
}
