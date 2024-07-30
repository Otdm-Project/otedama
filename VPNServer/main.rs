use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::fs;
use std::process::Command;
use std::collections::HashSet;

#[derive(Deserialize)]
struct PeerInfo {
    public_key: String,
    client_ip: String,
    endpoint: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct WgConfig {
    interface: Interface,
    peers: Vec<Peer>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Interface {
    private_key: String,
    address: String,
    listen_port: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Peer {
    public_key: String,
    allowed_ips: String,
    endpoint: String,
    persistent_keepalive: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let db_url = "http://<DBServerのIPアドレス>/api/peers";
    let private_key = std::fs::read_to_string("/home/vpnuser/privatekey")?.trim().to_string();
    let vpn_server_ip = "<VPNサーバのIP>";

    let mut known_peers = HashSet::new();

    loop {
        let res: Vec<PeerInfo> = client.get(db_url).send().await?.json().await?;

        let mut new_peers = Vec::new();

        for peer in res {
            let peer_config = Peer {
                public_key: peer.public_key.clone(),
                allowed_ips: format!("{}/32", peer.client_ip),
                endpoint: peer.endpoint.clone(),
                persistent_keepalive: 25,
            };

            if known_peers.insert(peer.public_key.clone()) {
                new_peers.push(peer_config);
            }
        }

        if !new_peers.is_empty() {
            update_wireguard_config(&private_key, vpn_server_ip, &new_peers).await?;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}

async fn update_wireguard_config(private_key: &str, vpn_server_ip: &str, new_peers: &[Peer]) -> Result<(), Box<dyn std::error::Error>> {
    let conf_path = "/etc/wireguard/wg0.conf";
    
    let mut wg_config = WgConfig {
        interface: Interface {
            private_key: private_key.to_string(),
            address: format!("{}/24", vpn_server_ip),
            listen_port: 51820,
        },
        peers: Vec::new(),
    };

    if let Ok(contents) = fs::read_to_string(conf_path).await {
        wg_config = toml::from_str(&contents)?;
    }

    wg_config.peers.extend_from_slice(new_peers);
    
    let new_config = toml::to_string(&wg_config)?;

    fs::write(conf_path, new_config).await?;

    Command::new("sudo")
        .arg("wg-quick")
        .arg("down")
        .arg("wg0")
        .output()
        .expect("Failed to bring down WireGuard interface");

    Command::new("sudo")
        .arg("wg-quick")
        .arg("up")
        .arg("wg0")
        .output()
        .expect("Failed to bring up WireGuard interface");

    Ok(())
}
