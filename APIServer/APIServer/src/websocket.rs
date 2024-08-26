use warp::Filter;
use warp::ws::{Message, WebSocket};
use futures_util::{StreamExt, SinkExt}; // StreamExtトレイトをインポート
use std::net::Ipv4Addr;

// WebSocket接続を処理する関数
async fn handle_socket(ws: WebSocket) {
    // WebSocketの送受信用のチャネルに分割
    let (mut tx, mut rx) = ws.split();

    // メッセージを受信して処理するループ
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    let text = msg.to_str().unwrap();
                    println!("Received: {}", text);
                    // クライアントにメッセージを返す
                    tx.send(Message::text("Received your message")).await.unwrap();
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }
}

// WebSocketサーバーを起動する関数
pub async fn start_websocket_server() {
    // WebSocketのルートを設定
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handle_socket) // WebSocket接続時にhandle_socketを呼び出す
        });

    // サーバーをバインドして実行
    let ip_addr = Ipv4Addr::new(, , , );  // 例: Ipv4Addr::new(a, b, c, d) の形式で指定
    warp::serve(ws_route).run((ip_addr, 8080)).await;
}
