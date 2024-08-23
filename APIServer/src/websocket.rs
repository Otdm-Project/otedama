// websocket.rs

use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::accept_async;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;

/// WebSocketサーバーを起動し、顧客環境からの接続を待ち受けます。
/// 公開鍵を受信し、処理を行います。
pub async fn start_websocket_server() -> Result<(), Box<dyn std::error::Error>> {
    // サーバーが待ち受けるIPアドレスとポート番号
    let addr = "<APIServerのプライベートIPアドレス>".to_string();
    
    // TCPリスナーを指定したアドレスでバインドし、WebSocketサーバーを開始します
    let listener = TcpListener::bind(&addr).await?;
    println!("WebSocket server is listening on: {}", addr);

    // クライアントからの接続を待ち受けるループ
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            // 接続されたクライアントとのWebSocketストリームを確立します
            let ws_stream = accept_async(stream).await.expect("Failed to accept");

            println!("New WebSocket connection established");

            // WebSocketの読み取りと書き込みのストリームを分割します
            let (mut write, mut read) = ws_stream.split();

            // クライアントからのメッセージを待ち受け、処理を行います
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        println!("Received: {}", text);
                        
                        // 公開鍵を含むメッセージを受信した場合の処理
                        if text.starts_with("public_key:") {
                            let public_key = text.replace("public_key:", "");
                            println!("Public key received: {}", public_key);
                            // 公開鍵を変数に保管する（ここでは仮に保持するだけ）
                        }
                        
                        // クライアントに対して受信完了のメッセージを送信します
                        write.send(Message::Text("Key received".to_string())).await.unwrap();
                    }
                    Ok(_) => (),
                    Err(e) => println!("WebSocket error: {}", e),
                }
            }
        });
    }

    Ok(())
}
