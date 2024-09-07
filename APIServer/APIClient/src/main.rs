use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{StreamExt, SinkExt};
use url::Url;

#[tokio::main]
async fn main() {
    // WebSocketサーバーのURLを指定
    let url = Url::parse("ws://<APIServerのIPアドレス>:8080/ws").unwrap();

    // WebSocketサーバーに接続
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    // WebSocketストリームを送信と受信に分割
    let (mut write, mut read) = ws_stream.split();

    // 公開鍵を送信するタスク
    let send_task = tokio::spawn(async move {
        let msg = Message::text("pubkeytestpubkeytestpubkeytestpubkeytestpubkeytest");
        write.send(msg).await.expect("Failed to send message");
    });

    // メッセージを受信するタスク
    let receive_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            if msg.is_text() {
                println!("Received: {}", msg.to_text().unwrap());
            }
        }
    });

    // タスクを実行
    send_task.await.unwrap();
    receive_task.await.unwrap();
}
