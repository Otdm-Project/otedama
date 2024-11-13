use std::fs::{File, OpenOptions};
use std::io::{Write, Result};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use once_cell::sync::Lazy;

static IP_COUNTER: Lazy<Mutex<u32>> = Lazy::new(|| Mutex::new(1)); // 最初のクライアントIP (100.64.0.2)
const WG_CONFIG_FILE: &str = "/etc/wireguard/wg0.conf";
const PRIVATE_KEY_PATH: &str = "/etc/wireguard/privatekey";

// WireGuard設定ファイルの初期化
pub fn initialize_wg_config() {
    let private_key = generate_or_load_private_key();

    let config_content = format!(
        "[Interface]\n\
        Address = 100.64.0.1/10\n\
        SaveConfig = true\n\
        PostUp = iptables -A FORWARD -i wg0 -j ACCEPT; iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE\n\
        PostDown = iptables -D FORWARD -i wg0 -j ACCEPT; iptables -t nat -D POSTROUTING -o eth0 -j MASQUERADE\n\
        ListenPort = 51820\n\
        PrivateKey = {}\n",
        private_key
    );

    let mut file = File::create(WG_CONFIG_FILE).expect("Failed to create WireGuard config file");
    file.write_all(config_content.as_bytes()).expect("Failed to write config file");
}

// サーバの公開鍵を取得
pub fn get_server_public_key(private_key: &str) -> String {
    let mut pubkey_command = Command::new("wg")
        .arg("pubkey")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn wg pubkey command");

    {
        let stdin = pubkey_command.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(private_key.as_bytes()).expect("Failed to write to stdin");
    }

    let public_key_output = pubkey_command.wait_with_output().expect("Failed to read stdout");
    String::from_utf8_lossy(&public_key_output.stdout).trim().to_string()
}

// 秘密鍵の生成または読み込み
fn generate_or_load_private_key() -> String {
    if std::path::Path::new(PRIVATE_KEY_PATH).exists() {
        return std::fs::read_to_string(PRIVATE_KEY_PATH).expect("Failed to read private key");
    }

    let private_key = Command::new("wg")
        .arg("genkey")
        .output()
        .expect("Failed to generate private key")
        .stdout;

    let private_key_str = String::from_utf8_lossy(&private_key).trim().to_string();
    std::fs::write(PRIVATE_KEY_PATH, &private_key_str).expect("Failed to save private key");

    private_key_str
}

// クライアントIPの割り当て
pub fn allocate_ip_address() -> String {
    let mut counter = IP_COUNTER.lock().unwrap();
    if *counter >= (1 << 18) {
        panic!("No more IP addresses available in the /10 subnet");
    }

    let ip_octets = [
        100,
        64 + ((*counter >> 10) & 0x3F) as u8,
        ((*counter >> 2) & 0xFF) as u8,
        ((*counter & 0x03) + 2) as u8,
    ];

    *counter += 1;
    format!("{}.{}.{}.{}", ip_octets[0], ip_octets[1], ip_octets[2], ip_octets[3])
}

// Peer設定の追加
pub fn add_peer_to_config(client_public_key: &str, client_ip: &str) -> Result<()> {
    let peer_config = format!(
        "\n[Peer]\n\
        PublicKey = {}\n\
        AllowedIPs = {}/32\n",
        client_public_key, client_ip
    );

    let mut config_file = OpenOptions::new()
        .append(true)
        .open(WG_CONFIG_FILE)
        .expect("Failed to open WireGuard config file");

    config_file.write_all(peer_config.as_bytes())?;
    Ok(())
}
