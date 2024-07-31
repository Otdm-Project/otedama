use warp::{Reply, Rejection};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::{Arc, Mutex};

pub type SharedData<T> = Arc<Mutex<T>>;

#[derive(Deserialize, Serialize)]
struct CreateTunnelRequest {
    public_key: String,
}

struct TunnelConfig {
    client_priv_key: String,
    client_pub_key: String,
    server_pub_key: String,
    client_ip: String,
    server_ip: String,
}

// トンネル生成指示受信
pub async fn handle_create_tunnel(body: CreateTunnelRequest) -> Result<impl Reply, Rejection> {
    let CreateTunnelRequest { public_key } = body;

    // キーペア作成等の処理
    let client_priv_key = genkey();
    let client_pub_key = pubkey(&client_priv_key);
    let server_priv_key = genkey(); // サーバ側は固定できる場合はここで生成しない
    let server_pub_key = pubkey(&server_priv_key);

    let client_ip = "10.0.0.2".to_string();
    let server_ip = "10.0.0.1".to_string();

    let config = TunnelConfig {
        client_priv_key,
        client_pub_key,
        server_pub_key,
        client_ip: client_ip.clone(),
        server_ip,
    };

    // Peerインターフェイス設定
    add_peer(&public_key, &client_ip).unwrap();

    // APIServerに報告
    // (これは実際のシナリオに応じて変更可能)
    
    Ok(warp::reply::json(&config))
}

// 外部コマンドを利用して公開鍵生成
fn genkey() -> String {
    let output = Command::new("wg")
        .arg("genkey")
        .output()
        .expect("Failed to generate private key");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn pubkey(priv_key: &str) -> String {
    let output = Command::new("wg")
        .arg("pubkey")
        .arg(priv_key)
        .output()
        .expect("Failed to generate public key");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// Peerインターフェイス設定
fn add_peer(public_key: &str, client_ip: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("wg")
        .arg("set")
        .arg("wg0")
        .arg("peer")
        .arg(public_key)
        .arg(format!("allowed-ips={}/32", client_ip))
        .output()?;
    if !output.status.success() {
        eprintln!("wg set peer failed: {}", String::from_utf8_lossy(&output.stderr));
        return Err("wg set peer failed".into());
    }
    Ok(())
}
