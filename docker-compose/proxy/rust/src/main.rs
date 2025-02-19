mod handler;
mod subdomain;
mod db;

use warp::Filter;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // WebSocketサーバーを非同期タスクで起動
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handler::handle_socket)
        });

    let addr = "0.0.0.0:8100".parse::<SocketAddr>().expect("Unable to parse socket address");
    println!("ProxyServer is running on {}", addr);

    tokio::spawn(async move {
        warp::serve(ws_route).run(addr).await;
    });

    // メイン関数が終了しないように待機
    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
}

