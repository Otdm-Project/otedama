use std::sync::Mutex;
use std::io::{Result, Write};
use std::fs::OpenOptions;
use std::process::Command;
use once_cell::sync::Lazy;
use tracing::{info, error};

static DOMAIN: &str = "otdm.dev";
static CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

// サブドメイン生成用のカウンタ
static SUBDOMAIN_COUNTER: Lazy<Mutex<Vec<usize>>> = Lazy::new(|| Mutex::new(vec![0; 5]));

// サブドメイン生成
pub fn generate_subdomain() -> Result<String> {
    let mut counter = SUBDOMAIN_COUNTER.lock().unwrap();
    let mut subdomain = String::new();

    for &index in counter.iter() {
        subdomain.push(CHARSET[index] as char);
    }

    let mut i = counter.len() - 1;
    loop {
        counter[i] += 1;
        if counter[i] < CHARSET.len() {
            break;
        }
        counter[i] = 0;
        if i == 0 {
            counter.insert(0, 0);
            break;
        } else {
            i -= 1;
        }
    }

    let full_domain = format!("{}.{}", subdomain, DOMAIN);
    info!("Generated subdomain: {}", full_domain);

    Ok(full_domain)
}

/// サブドメインを生成し、HAProxyに追加する
pub fn generate_and_add_subdomain(client_ip: &str) -> Result<String> {
    let subdomain = generate_subdomain()?;
    info!("サブドメイン:{} ClientIP:{}", subdomain, client_ip);
    add_server_to_haproxy(&subdomain, client_ip)?;
    Ok(subdomain)
}

/// HAProxy設定ファイルにサーバエントリを追加
pub fn add_server_to_haproxy(subdomain: &str, client_ip: &str) -> Result<()> {
    let formatted_subdomain = subdomain.replace('.', "_");
    let new_server_entry = format!(" server {} {}:80 check\n", formatted_subdomain, client_ip);
    let haproxy_config = "/etc/haproxy/haproxy.cfg";

    let mut file = OpenOptions::new()
        .append(true)
        .open(haproxy_config)?;

    file.write_all(new_server_entry.as_bytes())?;
    info!("Added server to HAProxy config: {}", new_server_entry);

    let output = Command::new("systemctl")
        .arg("reload")
        .arg("haproxy")
        .output()?;
    if !output.status.success() {
        error!("Failed to reload HAProxy: {}", String::from_utf8_lossy(&output.stderr));
    } else {
        info!("HAProxy reloaded to apply new configuration.");
    }

    Ok(())
}
