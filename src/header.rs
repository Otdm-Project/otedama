use warp::ws::Message;
use warp::{Reply, Rejection};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

pub type SharedData<T> = Arc<Mutex<T>>;

#[derive(Deserialize, Serialize)]
struct PublicKeyRequest {
    public_key: String,
}

// 通信トンネルの管理共有データ
struct TunnelData {
    public_key: Option<String>,
}

// WebSocket待受
pub async fn handle_websocket(ws: warp::ws::Ws) -> Result<impl Reply, Rejection> {
    let data = Arc::new(Mutex::new(TunnelData { public_key: None }));
    Ok(ws.on_upgrade(move |websocket| handle_websocket_connection(websocket, data)))
}

async fn handle_websocket_connection(ws: warp::ws::WebSocket, data: SharedData<TunnelData>) {
    let (mut tx, mut rx) = ws.split();

    while let Some(Ok(message)) = rx.next().await {
        if message.is_text() {
            let msg_text = message.to_str().unwrap();
            println!("Received WebSocket Message: {}", msg_text);
            // ここでメッセージを処理し、トンネル生成や公開鍵の記録を行う
        }
    }
}

// 公開鍵待受
pub async fn handle_public_key(body: PublicKeyRequest) -> Result<impl Reply, Rejection> {
    println!("Received public key: {:?}", body.public_key);
    // 公開鍵を変数に記録
    // DBに公開鍵を送信
    // VPNServerにトンネル生成指示
    Ok(warp::reply::json(&"Public key received"))
}
