use warp::Filter;
use warp::ws::{Message, WebSocket};
use futures_util::{StreamExt, SinkExt}; // StreamExtとSinkExtをインポート

async fn handle_socket(ws: WebSocket) {
    let (mut tx, mut rx) = ws.split();
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    let text = msg.to_str().unwrap();
                    println!("Received tunnel creation request for Customer ID: {}", text);

                    // ここにWireGuardトンネル生成のコードを追加する予定です

                    tx.send(Message::text("Tunnel creation initiated")).await.unwrap();
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handle_socket)
        });

        let addr: std::net::SocketAddr = "0.0.0.0:8090".parse().expect("Unable to parse socket address");
        warp::serve(ws_route).run(addr).await;

}
