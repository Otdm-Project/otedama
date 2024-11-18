use std::sync::Mutex;
use std::io::{Result, Write};
use std::fs::OpenOptions;
use std::process::Command;

static DOMAIN: &str = "otdm.dev";
static CHARSET: &str = "abcdefghijklmnopqrstuvwxyz0123456789"; // ドメインに使用するアルファベットと数字の組み合わせ

lazy_static::lazy_static! {
    static ref SUBDOMAIN_COUNTER: Mutex<Vec<usize>> = Mutex::new(vec![0; 5]); // 初期は5文字のカウンタ
}

// サブドメイン生成
pub fn generate_subdomain() -> Result<String> {
    let mut counter = SUBDOMAIN_COUNTER.lock().unwrap();
    let mut subdomain = String::new();

    for &index in counter.iter() {
        subdomain.push(CHARSET.chars().nth(index).unwrap());
    }

    counter[0] += 1;
    let mut i = 0;
    while i < counter.len() && counter[i] == CHARSET.len() {
        counter[i] = 0;
        if i + 1 < counter.len() {
            counter[i + 1] += 1;
        } else {
            counter.push(0); // 必要に応じて文字数を増加
        }
        i += 1;
    }

    let full_domain = format!("{}.{}", subdomain, DOMAIN);
    println!("Generated subdomain: {}", full_domain);

    Ok(full_domain)
}

// HAProxy設定ファイルにサーバエントリを追加
pub fn add_server_to_haproxy(subdomain: &str, client_ip: &str) -> Result<()> {
    // サーバ名を生成（サブドメインのドットをアンダースコアに置換）
    let server_name = format!("{}", subdomain.replace(".", "_")); // abcde_otdm_devのように生成


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
    let subdomain = generate_subdomain()?;
    add_server_to_haproxy(&subdomain, client_ip)?;
    Ok(subdomain)
}
