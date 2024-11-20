use std::sync::Mutex;
use std::io::{Result, Write};
use std::fs::OpenOptions;
use std::process::Command;
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

// HAProxy設定ファイルにサーバエントリを追加
pub fn add_server_to_haproxy(subdomain: &str, client_ip: &str) -> Result<()> {
    // サーバ名を生成（サブドメインのドットをアンダースコアに置換）
    let server_name = format!("{}", subdomain.replace(".", "_"));
    
    // HAProxyのbackendセクションにサーバエントリを追加
    let new_server_entry = format!("    server {} {}:80 check\n", server_name, client_ip);

    let mut file = OpenOptions::new()
        .append(true)
        .open("/etc/haproxy/haproxy.cfg")?;

    file.write_all(new_server_entry.as_bytes())?;

    println!("Added server to HAProxy config: {}", new_server_entry);

    // HAProxyの再読み込み
    Command::new("systemctl")
        .arg("reload")
        .arg("haproxy")
        .output()?;

    println!("HAProxy reloaded to apply new configuration.");

    Ok(())
}

// サブドメインを生成し、HAProxyに追加する
pub fn generate_and_add_subdomain(client_ip: &str) -> Result<String> {
    let subdomain = generate_subdomain()?; // 修正: `Result<String>` を期待
    add_server_to_haproxy(&subdomain, client_ip)?;
    Ok(subdomain)
}
