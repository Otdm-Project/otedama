use warp::Filter;
use std::net::SocketAddr;
use std::process::Command;
use std::io::{Result, Write};
use warp::ws::{Message, WebSocket};
use futures_util::{StreamExt, SinkExt};

// DBServerから顧客環境の公開鍵を取得
pub fn get_public_key(customer_id: usize) -> Result<String> {
    let query = format!("SELECT client_public_key FROM customer_data.customer_info WHERE customer_id = {};", customer_id);

    println!("Executing query: {}", query);

    let output = Command::new("cqlsh")
        .arg("10.0.10.40")
        .arg("-e")
        .arg(query)
        .output()?;

    let public_key_output = String::from_utf8_lossy(&output.stdout).to_string();

    // 出力の行を取得
    let mut lines = public_key_output.lines();

    // データ行まで進むためにヘッダーと区切り線をスキップ
    while let Some(line) = lines.next() {
        if line.trim().starts_with('-') {
            break; // 区切り線に到達したら次からデータ行
        }
    }

    // 次の行がデータ行であることを期待して取得
    if let Some(public_key_line) = lines.next() {
        let public_key = public_key_line.trim().to_string();
        println!("Extracted public key: {}", public_key);
        Ok(public_key)
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Public key not found"))
    }
}

// DBServerにPeer設定用の情報を送信
pub fn insert_tunnel_data(customer_id: usize, server_public_key: &str, client_public_key: &str, client_ip: &str, server_ip: &str) -> Result<()> {
    let insert_query = format!(
        "UPDATE customer_data.customer_info SET server_public_key = '{}', client_public_key = '{}', vpn_ip_client = '{}', vpn_ip_server = '{}' WHERE customer_id = {};",
        server_public_key, client_public_key, client_ip, server_ip, customer_id
    );

    println!("Executing insert query: {}", insert_query);

    Command::new("cqlsh")
        .arg("10.0.10.40")
        .arg("-e")
        .arg(insert_query)
        .output()?;

    Ok(())
}

pub async fn handle_socket(ws: WebSocket) {
    let (mut tx, mut rx) = ws.split();
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    let text = msg.to_str().unwrap();
                    println!("Received tunnel creation request for Customer ID: {}", text);
                    let customer_id: usize = text.parse().unwrap_or(0);
                    if customer_id > 0 {
                        match get_public_key(customer_id) {
                            Ok(client_public_key) => {
                                println!("Retrieved client public key: {}", client_public_key);

                                // サーバの公開鍵を取得
                                let private_key = std::fs::read_to_string("/etc/wireguard/privatekey")
                                    .expect("Failed to read server private key");
                                let server_public_key = get_server_public_key(&private_key);

                                // 仮想IPアドレスの割り当て
                                let client_ip = allocate_ip_address();
                                let server_ip = "100.64.0.1".to_string();

                                // DBにトンネルデータを保存
                                match insert_tunnel_data(customer_id, &server_public_key, &client_public_key, &client_ip, &server_ip) {
                                    Ok(_) => println!("Successfully inserted tunnel data into DB"),
                                    Err(e) => {
                                        eprintln!("Failed to insert tunnel data into DB: {}", e);
                                        tx.send(Message::text("Error saving tunnel data")).await.unwrap();
                                    }
                                }

                                // WireGuardのPeer設定の追加
                                match add_peer_to_wireguard(&client_public_key, &client_ip) {
                                    Ok(_) => println!("Successfully added peer to WireGuard config"),
                                    Err(e) => {
                                        eprintln!("Failed to add peer to WireGuard config: {}", e);
                                        tx.send(Message::text("Error adding peer to config")).await.unwrap();
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to retrieve public key: {}", e);
                                tx.send(Message::text("Error retrieving public key")).await.unwrap();
                            }
                        }
                    }
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

// WireGuard設定ファイルの初期化
pub fn initialize_wg_config() {
    let private_key = generate_or_load_private_key();
    let config_content = format!(
        "[Interface]\n\
        Address = 100.64.0.1/10\n\
        ListenPort = 51820\n\
        PrivateKey = {}\n",
        private_key
    );

    std::fs::write("/etc/wireguard/wg0.conf", config_content).expect("Failed to create WireGuard config file");
    println!("WireGuard config initialized.");
}

// WireGuardにPeerを動的に追加し、wg0.confに追記
pub fn add_peer_to_wireguard(public_key: &str, client_ip: &str) -> Result<()> {
    // Peerを動的に追加
    let output = Command::new("wg")
        .arg("set")
        .arg("wg0")
        .arg("peer")
        .arg(public_key)
        .arg("allowed-ips")
        .arg(format!("{}/32", client_ip))
        .output()?;
    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        eprintln!("Failed to add peer to WireGuard: {}", error_message);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, error_message));
    }
    println!("Successfully added peer to WireGuard: PublicKey = {}, AllowedIPs = {}/32", public_key, client_ip);
    Ok(())
}

// 秘密鍵の生成または読み込み
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

// 仮想IPアドレスの割り当て
static mut COUNTER: u32 = 2;
pub fn allocate_ip_address() -> String {
    unsafe {
        let ip = format!("100.64.{}.{}", (COUNTER >> 8) & 0xFF, COUNTER & 0xFF);
        COUNTER += 1;
        ip
    }
}

// サーバ公開鍵の取得
pub fn get_server_public_key(private_key: &str) -> String {
    let mut process = Command::new("wg")
        .arg("pubkey")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to execute 'wg pubkey' command");

    {
        // 標準入力に秘密鍵を送信
        let stdin = process.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(private_key.as_bytes()).expect("Failed to write private key to stdin");
    }

    let output = process
        .wait_with_output()
        .expect("Failed to read 'wg pubkey' output");
    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        eprintln!("Failed to generate public key: {}", error_message);
        panic!("Failed to generate public key");
    }

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}


#[tokio::main]
async fn main() {
    // WireGuard設定ファイルの初期化
    initialize_wg_config();

    // WebSocketサーバーを非同期タスクで起動
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handle_socket)
        });

    let addr: SocketAddr = "0.0.0.0:8090".parse().expect("Unable to parse socket address");
    println!("VPNServer is running on {}", addr);

    tokio::spawn(async move {
        warp::serve(ws_route).run(addr).await;
    });

    // メイン関数が終了しないように待機
    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
}