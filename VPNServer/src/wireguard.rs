use std::fs::{self};
use std::io::{Result, Write};
use std::process::Command;

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
        eprintln!("Failed to add peer to WireGuard: {}", error_message);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, error_message));
    }

    println!("Successfully added peer to WireGuard: PublicKey = {}, AllowedIPs = {}/32", public_key, client_ip);

    // 設定をwg0.confに追記
    append_peer_to_config(public_key, client_ip)?;

    Ok(())
}

// wg0.confに新しいPeerを追記
fn append_peer_to_config(public_key: &str, client_ip: &str) -> Result<()> {
    // 既存の設定を保持して読み込む
    let mut config = fs::read_to_string("/etc/wireguard/wg0.conf").expect("Failed to read WireGuard config");

    // 新しいPeer情報を追加
    let peer_entry = format!(
        "\n[Peer]\n\
        PublicKey = {}\n\
        AllowedIPs = {}/32\n",
        public_key, client_ip
    );

    config.push_str(&peer_entry);

    // ファイル全体を書き直す
    std::fs::write("/etc/wireguard/wg0.conf", config)?;

    println!("Successfully appended peer to wg0.conf: {}", peer_entry.trim());
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
