mod monitoring;

use warp::Filter;
use warp::ws::{Message, WebSocket};
use futures_util::{StreamExt, SinkExt};
use std::process::Command;
use std::io::{Result, Write};
use std::net::SocketAddr;
use tracing::{info, error};

#[tokio::main]
async fn main() {
    // ログの初期化
    tracing_subscriber::fmt()
        .with_target(false)
        .init();

    // WireGuard設定ファイルの初期化
    initialize_wg_config();

    // WebSocketサーバーを非同期タスクで起動
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handle_socket)
        });

    let addr: SocketAddr = "0.0.0.0:8090".parse().expect("Unable to parse socket address");
    info!("VPNServer is running on {}", addr);

    tokio::spawn(async move {
        warp::serve(ws_route).run(addr).await;
    });

    // monitoring関数を別スレッドで非同期タスクとして実行
    tokio::task::spawn_blocking(|| {
        info!("Starting monitoring...");
        monitoring();
    })
    .await
    .expect("Failed to run monitoring");

    // メイン関数が終了しないように待機
    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
}

/// WireGuardの設定ファイルを初期化
fn initialize_wg_config() {
    let private_key = generate_or_load_private_key();
    let config_content = format!(
        "[Interface]\nAddress = 100.64.0.1/10\nListenPort = 51820\nPrivateKey = {}\n",
        private_key
    );

    std::fs::write("/etc/wireguard/wg0.conf", config_content).expect("Failed to create WireGuard config file");
    info!("WireGuard config initialized.");
}

/// WebSocketリクエストを処理する
async fn handle_socket(ws: WebSocket) {
    let (mut tx, mut rx) = ws.split();
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    let text = msg.to_str().unwrap();
                    info!("Received tunnel creation request for Customer ID: {}", text);
                    let customer_id: usize = text.parse().unwrap_or(0);
                    if customer_id > 0 {
                        match get_public_key(customer_id) {
                            Ok(client_public_key) => {
                                info!("Retrieved client public key: {}", client_public_key);

                                // サーバの公開鍵を取得
                                let private_key = std::fs::read_to_string("/etc/wireguard/privatekey")
                                    .expect("Failed to read server private key");
                                let server_public_key = get_server_public_key(&private_key);

                                // 仮想IPアドレスの割り当て
                                let client_ip = allocate_ip_address();
                                let server_ip = "100.64.0.1".to_string();

                                // DBにトンネルデータを保存
                                match insert_tunnel_data(customer_id, &server_public_key, &client_public_key, &client_ip, &server_ip) {
                                    Ok(_) => info!("Successfully inserted tunnel data into DB for Customer ID: {}", customer_id),
                                    Err(e) => {
                                        error!("Failed to insert tunnel data into DB for Customer ID: {}: {}", customer_id, e);
                                        tx.send(Message::text("Error saving tunnel data")).await.unwrap();
                                    }
                                }

                                // WireGuardのPeer設定の追加
                                match add_peer_to_wireguard(&client_public_key, &client_ip) {
                                    Ok(_) => info!("Successfully added peer to WireGuard config for Customer ID: {}", customer_id),
                                    Err(e) => {
                                        error!("Failed to add peer to WireGuard config for Customer ID: {}: {}", customer_id, e);
                                        tx.send(Message::text("Error adding peer to config")).await.unwrap();
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to retrieve public key for Customer ID {}: {}", customer_id, e);
                                tx.send(Message::text("Error retrieving public key")).await.unwrap();
                            }
                        }
                    } else {
                        error!("Invalid customer_id received: {}", text);
                        tx.send(Message::text("Invalid Customer ID")).await.unwrap();
                    }
                }
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    if let Err(e) = tx.close().await {
        error!("Failed to close WebSocket connection: {}", e);
    }
}

/// DBから顧客の公開鍵を取得
fn get_public_key(customer_id: usize) -> Result<String> {
    let query = format!("SELECT client_public_key FROM customer_data.customer_info WHERE customer_id = {};", customer_id);

    info!("Executing query: {}", query);

    let output = Command::new("/home/vpnuser/.local/bin/cqlsh")
        .arg("10.0.10.40")
        .arg("-e")
        .arg(&query)
        .output()?;

    let public_key_output = String::from_utf8_lossy(&output.stdout).to_string();

    let mut lines = public_key_output.lines();
    while let Some(line) = lines.next() {
        if line.trim().starts_with('-') {
            break;
        }
    }

    if let Some(public_key_line) = lines.next() {
        let public_key = public_key_line.trim().to_string();
        info!("Extracted public key: {}", public_key);
        Ok(public_key)
    } else {
        error!("Public key not found for customer_id: {}", customer_id);
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Public key not found"))
    }
}

/// DBにトンネルデータを保存
fn insert_tunnel_data(customer_id: usize, server_public_key: &str, client_public_key: &str, client_ip: &str, server_ip: &str) -> Result<()> {
    let insert_query = format!(
        "UPDATE customer_data.customer_info SET server_public_key = '{}', client_public_key = '{}', vpn_ip_client = '{}', vpn_ip_server = '{}' WHERE customer_id = {};",
        server_public_key, client_public_key, client_ip, server_ip, customer_id
    );

    info!("Executing insert query: {}", insert_query);

    let output = Command::new("/home/vpnuser/.local/bin/cqlsh")
        .arg("10.0.10.40")
        .arg("-e")
        .arg(&insert_query)
        .output()?;

    if !output.status.success() {
        error!("Failed to insert tunnel data for customer_id: {}", customer_id);
    } else {
        info!("Successfully inserted tunnel data for customer_id: {}", customer_id);
    }

    Ok(())
}

/// WireGuardにPeerを動的に追加
fn add_peer_to_wireguard(public_key: &str, client_ip: &str) -> Result<()> {
    let output = Command::new("sudo")
        .arg("wg")
        .arg("set")
        .arg("wg0")
        .arg("peer")
        .arg(public_key)
        .arg("allowed-ips")
        .arg(format!("{}/32", client_ip))
        .output()?;
    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        error!("Failed to add peer to WireGuard: {}", error_message);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, error_message.to_string()));
    }
    info!("Successfully added peer to WireGuard: PublicKey = {}, AllowedIPs = {}/32", public_key, client_ip);
    Ok(())
}

/// 仮想IPアドレスを割り当てる
static mut COUNTER: u32 = 2;
fn allocate_ip_address() -> String {
    unsafe {
        let ip = format!("100.64.{}.{}", (COUNTER >> 8) & 0xFF, COUNTER & 0xFF);
        COUNTER += 1;
        ip
    }
}

/// サーバ公開鍵を取得
fn get_server_public_key(private_key: &str) -> String {
    let mut process = Command::new("wg")
        .arg("pubkey")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to execute 'wg pubkey' command");

    {
        let stdin = process.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(private_key.as_bytes()).expect("Failed to write private key to stdin");
    }

    let output = process
        .wait_with_output()
        .expect("Failed to read 'wg pubkey' output");
    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        error!("Failed to generate public key: {}", error_message);
        panic!("Failed to generate public key");
    }

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// 秘密鍵を生成または読み込む
fn generate_or_load_private_key() -> String {
    let private_key_path = "/etc/wireguard/privatekey";

    if let Ok(private_key) = std::fs::read_to_string(private_key_path) {
        private_key
    } else {
        let private_key = Command::new("wg")
            .arg("genkey")
            .output()
            .expect("Failed to generate private key")
            .stdout;
        let private_key_str = String::from_utf8_lossy(&private_key).trim().to_string();
        std::fs::write(private_key_path, &private_key_str).expect("Failed to save private key");
        private_key_str
    }
}

/// 監視を開始する
fn monitoring() {
    let client_handle = std::thread::spawn(|| {
        info!("monitoring C start!");
        monitoring::start_client();
    });

    client_handle.join().unwrap();
}
