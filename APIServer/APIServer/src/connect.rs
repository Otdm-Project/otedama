use warp::Filter;
use warp::ws::{Message, WebSocket};
use std::sync::atomic::{AtomicUsize, Ordering};
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
use url::Url;
// IDに使用する値のカウンタ
static ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

async fn handle_socket(ws: WebSocket) {
    let (mut tx, mut rx) = ws.split();
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    let text = msg.to_str().unwrap();
                    if text.contains("INSERT completed for Customer ID") {
                        println!("Received notification from VPNServer: {}", text);

                        let customer_id: usize = text.split_whitespace().last().unwrap().parse().unwrap();
                        send_subdomain_creation_request(customer_id).await;
                    } else {
                        println!("Received: {}", text);

                        let id = ID_COUNTER.fetch_add(1, Ordering::SeqCst);
                        send_to_db(id, text).expect("Failed to send data to DB");

                        // VPNServerへのトンネル生成の指示
                        send_tunnel_creation_request(id).await;
                    }
                    
                    tx.send(Message::text("Operation completed")).await.unwrap();
                }
            }
            // エラーならその内容を出力
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // WebSocket接続を適切に終了
    if let Err(e) = tx.close().await {
        eprintln!("Failed to close WebSocket connection: {}", e);
    }
}

// DBServerに対して生成したID,受領した公開鍵をINSERT
fn send_to_db(id: usize, public_key: &str) -> std::io::Result<()> {
    // INSERT用のフォーマットを定義し、引数を用いてCQLSHでINSERTできるように
    let insert_query = format!(
        "INSERT INTO customer_data.customer_info (customer_id, client_public_key) VALUES ({}, '{}');",
        id, public_key
    );
    std::process::Command::new("cqlsh")
        .arg("<DBServerのIPアドレス>")
        .arg("-e")
        .arg(insert_query)
        .output()?;
    Ok(())
}

// VPNServerにIDとトンネル生成指示
async fn send_tunnel_creation_request(customer_id: usize) {
    // WebSocketトンネルを確立
    let url = Url::parse("ws://<VPNServerのIPアドレス>:8090/ws").unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect to VPNServer");
    let (mut write, _) = ws_stream.split();
    // IDを送信
    let msg = TungsteniteMessage::text(customer_id.to_string());
    write.send(msg).await.expect("Failed to send customer ID");
    println!("Sent tunnel creation request for Customer ID: {}", customer_id);
}

// ProxyServerにサブドメイン生成指示
async fn send_subdomain_creation_request(customer_id: usize) {
    let url = Url::parse("ws://<ProxyServerのIPアドレス>:8100/ws").unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect to ProxyServer");

    let (mut write, _) = ws_stream.split();

    let msg = TungsteniteMessage::text(customer_id.to_string());
    write.send(msg).await.expect("Failed to send subdomain creation request");

    println!("Sent subdomain creation request for Customer ID: {}", customer_id);
}

// WebSocket待受処理
pub async fn start_websocket_server() {
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handle_socket)
        });

    let addr = "0.0.0.0:8080".parse::<std::net::SocketAddr>().expect("Unable to parse socket address");
    warp::serve(ws_route).run(addr).await;
}
