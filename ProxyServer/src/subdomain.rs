use std::io::{Result, Write, Read};
use std::net::TcpStream;
use std::sync::Mutex;

use once_cell::sync::Lazy;

static DOMAIN: &str = "otdm.dev";
static CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789"; // 使用する文字セット

// サブドメイン生成用のカウンタ
static SUBDOMAIN_COUNTER: Lazy<Mutex<Vec<usize>>> = Lazy::new(|| Mutex::new(vec![0; 5]));

// サブドメイン生成
pub fn generate_subdomain() -> Result<String> {
    let mut counter = SUBDOMAIN_COUNTER.lock().unwrap();
    let mut subdomain = String::new();

    // 現在のカウンタに基づいてサブドメインを生成
    for &index in counter.iter() {
        subdomain.push(CHARSET[index] as char);
    }

    // カウンタをインクリメント（右端から進む）
    let mut i = counter.len() - 1;
    loop {
        counter[i] += 1;

        // 繰り上げ処理
        if counter[i] < CHARSET.len() {
            break;
        }
        counter[i] = 0; // 繰り上げた位置をリセット

        // 左側に繰り上げを伝播
        if i == 0 {
            // 最左端まで到達したら新しい桁を追加
            counter.insert(0, 0);
            break;
        } else {
            i -= 1;
        }
    }

    // 完全なドメイン名を生成
    let full_domain = format!("{}.{}", subdomain, DOMAIN);
    println!("Generated subdomain: {}", full_domain);

    Ok(full_domain)
}

/// サブドメインを生成し、HAProxyに追加する
pub fn generate_and_add_subdomain(client_ip: &str) -> Result<String> {
    let subdomain = generate_subdomain()?;
    println!("subdomain:{}CvIP:{}", subdomain, client_ip);
    add_server_to_haproxy(&subdomain, client_ip)?;
    reload_haproxy()?;
    Ok(subdomain)
}

/// HAProxy設定ファイルにサーバエントリを追加
// HAProxy設定情報をまとめた構造体
struct HAProxyConfig {
    ip: &'static str,
    port: u16,
}

impl HAProxyConfig {
    // 新しいHAProxyConfigを作成
    fn new() -> Self {
        HAProxyConfig {
            ip: "10.0.10.31",
            port: 9999,
        }
    }

    // TCP接続を確立するメソッド
    fn connect(&self) -> std::io::Result<TcpStream> {
        TcpStream::connect((self.ip, self.port))
    }
}

/// HAProxy設定ファイルにサーバエントリを追加
pub fn add_server_to_haproxy(subdomain: &str, client_ip: &str) -> std::io::Result<()> {
    let config = HAProxyConfig::new();
    let formatted_subdomain = subdomain.replace('.', "_");

    let mut stream = config.connect()?;
    let use_backend_cmd = format!(
        "add server backend_{subdomain} {client_ip}:80 check\n",
        subdomain = formatted_subdomain,
        client_ip = client_ip
    );

    stream.write_all(use_backend_cmd.as_bytes())?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    println!("HAProxy response: {}", response);

    Ok(())
}

/// HAProxyの設定を再読み込み
pub fn reload_haproxy() -> std::io::Result<()> {
    let config = HAProxyConfig::new();

    let mut stream = config.connect()?;
    stream.write_all(b"reload\n")?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    println!("HAProxy reload response: {}", response);

    Ok(())
}