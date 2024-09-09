use std::process::{Command, Stdio}; 
use std::io::Write;
use std::sync::Mutex;
use once_cell::sync::Lazy;

static IP_COUNTER: Lazy<Mutex<u32>> = Lazy::new(|| Mutex::new(1)); // 最初のクライアントIP (100.64.0.2)

//WireGuard用のキーペアを作成
pub fn generate_keypair() -> (String, String) {
    let private_key_output = Command::new("wg")
        .arg("genkey")
        .output()
        .expect("Failed to generate private key");
    let private_key = String::from_utf8_lossy(&private_key_output.stdout).trim().to_string();

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

    let public_key = String::from_utf8_lossy(&public_key_output.stdout).trim().to_string();

    (private_key, public_key)
}

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

pub fn add_peer_to_config(client_public_key: &str, client_ip: &str) -> std::io::Result<()> {
    let status = Command::new("sudo")
        .arg("./add_peer.sh")
        .arg(client_public_key)
        .arg(client_ip)
        .status()?;

    if status.success() {
        println!("Peer information successfully added to config.");
    } else {
        eprintln!("Failed to add peer information to config.");
    }

    Ok(())
}

pub fn read_config() -> std::io::Result<String> {
    let output = Command::new("sudo")
        .arg("./read_config.sh")
        .output()?;

    let config_content = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(config_content)
}
