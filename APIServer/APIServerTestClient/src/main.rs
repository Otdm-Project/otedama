use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt}; // StreamExt をインポート

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // サーバーのWebSocket URL
    let url = url::Url::parse("ws://<APIServerのグローバル:IPアドレス>:8080").unwrap();

    // WebSocketサーバーに接続を確立します
    let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    // 送信する公開鍵のサンプルデータ
    let public_key = "public_key:example_key";

    // 公開鍵をWebSocketサーバーに送信します
    ws_stream.send(Message::Text(public_key.into())).await?;

    // サーバーからのメッセージを受信し、表示します
    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(Message::Text(text)) => println!("Received: {}", text),
            Ok(_) => (),
            Err(e) => println!("WebSocket error: {}", e),
        }
    }

    Ok(())
}
