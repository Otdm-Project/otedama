// src/subdomain.rs

use std::process::Command;
use std::io::{Result, Error, ErrorKind};
use std::fs::{read_to_string, OpenOptions};
use std::io::Write;
use std::sync::Mutex;
use once_cell::sync::Lazy;

// 定数の定義
static DOMAIN: &str = "otdma.net";
static CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789"; // 使用する文字セット

// サブドメイン生成用のカウンタ
static SUBDOMAIN_COUNTER: Lazy<Mutex<Vec<usize>>> = Lazy::new(|| Mutex::new(vec![0; 5]));

/// サブドメイン生成
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
pub fn add_server_to_haproxy(subdomain: &str, client_ip: &str) -> Result<()> {
    // ドメイン名内の `.` を `_` に置換
    let formatted_subdomain = subdomain.replace('.', "_");
    let haproxy_config_path = "/usr/local/etc/haproxy/haproxy.cfg";

    // 現在のHAProxy設定ファイルを読み込む
    let config = read_to_string(haproxy_config_path)?;

    // 1. frontend otdm_dev セクションを探して use_backend 行を追加
    let frontend_name = "frontend otdm_dev";
    let use_backend_line = format!(
        "    use_backend {} if {{ req.hdr(host) -i {} }}",
        formatted_subdomain, subdomain
    );

    // frontendセクションが存在するか確認
    if !config.contains(frontend_name) {
        eprintln!("No 'frontend otdm_dev' section found. Please define it before adding backends.");
        return Err(Error::new(
            ErrorKind::NotFound,
            "No 'frontend otdm_dev' section found",
        ));
    }

    // 設定ファイルを行ごとに分割
    let mut lines: Vec<String> = config.lines().map(|l| l.to_string()).collect();

    let mut frontend_start = None;
    let mut frontend_end = None;

    // frontendセクションの開始と終了を探す
    for (i, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with(frontend_name) {
            frontend_start = Some(i);
            continue;
        }
        if let Some(start) = frontend_start {
            // 次のセクションの開始でfrontendセクション終了とみなす
            if i > start
                && (line.trim_start().starts_with("frontend ")
                    || line.trim_start().starts_with("backend ")
                    || line.trim_start().starts_with("listen ")
                    || line.trim().is_empty())
            {
                frontend_end = Some(i);
                break;
            }
        }
    }

    // frontend_endが見つからない場合はファイル末尾がfrontendの終わり
    if frontend_start.is_some() && frontend_end.is_none() {
        frontend_end = Some(lines.len());
    }

    // frontendセクション内に既存のuse_backendがないか確認して、なければ挿入
    if let (Some(start), Some(end)) = (frontend_start, frontend_end) {
        let existing_use_backend = lines[start..end]
            .iter()
            .any(|l| l.trim() == use_backend_line.trim());
        if !existing_use_backend {
            // frontendセクション末尾へ挿入
            lines.insert(end, use_backend_line.clone());
        }
    }

    // 2. 新しいbackendセクションを追加
    let backend_header = format!("backend {}", formatted_subdomain);
    if !lines.iter().any(|l| l.trim() == backend_header) {
        // 末尾にbackendセクションを追加
        lines.push("".to_string());
        lines.push(backend_header);
        lines.push("    mode http".to_string());
        lines.push("    balance roundrobin".to_string());
        let server_line = format!("    server {} {}:80 check", formatted_subdomain, client_ip);
        lines.push(server_line);
    }

    // 修正済みconfigを書き戻し
    let new_config = lines.join("\n") + "\n";
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(haproxy_config_path)?;
    file.write_all(new_config.as_bytes())?;

    println!("Added server and backend to HAProxy config.");

    Ok(())
}

/// HAProxyの設定を再読み込み
pub fn reload_haproxy() -> Result<()> {
    // HAProxyを再読み込みするdocker execコマンド
    // docker-composeでのサービス名が "haproxy" であることを確認してください
    let reload_command = "haproxy -f /usr/local/etc/haproxy/haproxy.cfg -sf $(pidof haproxy)";

    let output = Command::new("docker")
        .arg("exec")
        .arg("docker-compose-haproxy-1") // docker-composeでのサービス名
        .arg("sh")
        .arg("-c")
        .arg(reload_command)
        .output()?;

    if !output.status.success() {
        eprintln!(
            "HAProxy reload failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(Error::new(
            ErrorKind::Other,
            "Failed to reload HAProxy via docker exec",
        ));
    }

    println!("HAProxy reloaded to apply new configuration.");

    Ok(())
}
