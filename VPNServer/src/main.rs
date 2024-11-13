mod db;
mod handler;
mod wireguard;
mod monitoring;

use warp::Filter;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // WireGuard設定ファイルの初期化
    wireguard::initialize_wg_config();

    // WebSocketサーバーを非同期タスクで起動
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handler::handle_socket)
        });

    let addr: SocketAddr = "0.0.0.0:8090".parse().expect("Unable to parse socket address");
    println!("VPNServer is running on {}", addr);

    tokio::spawn(async move {
        warp::serve(ws_route).run(addr).await;
    });

    // monitoring関数を別スレッドで非同期タスクとして実行
    tokio::task::spawn_blocking(|| {
        println!("Starting monitoring...");
        monitoring();
    })
    .await
    .expect("Failed to run monitoring");

    // メイン関数が終了しないように待機
    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
}

// monitoring関数を呼び出すために定義
fn monitoring() {
    // クライアントを起動
    let client_handle = std::thread::spawn(|| {
        println!("monitoring C start!");
        monitoring::start_client();
    });

    // 両スレッドを終了まで待機
    client_handle.join().unwrap();
}
