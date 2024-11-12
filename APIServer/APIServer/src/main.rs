use warp::Filter;
use warp::ws::{Message, WebSocket};
use std::sync::atomic::{AtomicUsize, Ordering};
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
use url::Url;
use std::process::Command;
use std::thread;
use serde_json::json;
use serde_json;

mod monitoring;


// IDに使用する値のカウンタ
static ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[tokio::main]
async fn main() {
    println!("Starting WebSocket server...");

    // WebSocketサーバーを非同期タスクで起動
    tokio::spawn(async {
        start_websocket_server().await;
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

async fn handle_socket(ws: WebSocket) {
    let (mut tx, mut rx) = ws.split();
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    println!("実行！");
                    let text = msg.to_str().unwrap();
                    println!("Received: {}", text);
                    
                    // DBにデータを送信
                    let id = ID_COUNTER.fetch_add(1, Ordering::SeqCst);
                    println!("{},{}",id,text);
                    if let Err(e) = send_to_db(id, text) {
                        eprintln!("Failed to send data to DB: {:?}", e);
                        let error_message = json!({
                            "status": "error",
                            "message": "データ送信に失敗しました"
                        });
                        tx.send(Message::text(error_message.to_string())).await.unwrap();
                        continue;
                    }
                    
                    println!("Received notification from VPNServer: {}", text);
                    // トンネルとサブドメインの生成リクエストを送信
                    send_tunnel_creation_request(id).await;
                    send_subdomain_creation_request(id).await;
                    
                    // DBから顧客情報を取得して応答
                    if let Some(info) = retrieve_customer_info_from_db(id) {
                        let response = serde_json::to_string(&info).expect("Failed to serialize customer info");
                        
                        // 顧客情報のメッセージ送信
                        if let Err(e) = tx.send(Message::text(response)).await {
                            eprintln!("Failed to send customer info: {:?}", e);
                        } else {
                            println!("Customer info sent successfully");
                        }
                    } else {
                        let error_message = json!({
                            "status": "error",
                            "message": "顧客情報の取得に失敗しました"
                        });
                        tx.send(Message::text(error_message.to_string())).await.unwrap();
                    }
                    
                    // メッセージが正常に処理されたことを通知
                    let success_message = json!({
                        "status": "success",
                        "message": "Operation completed"
                    });
                    tx.send(Message::text(success_message.to_string())).await.unwrap();
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }

    if let Err(e) = tx.close().await {
        eprintln!("Failed to close WebSocket connection: {}", e);
    }
}
