use std::{
    fs::{read_to_string, OpenOptions},
    io::{Result, Error, ErrorKind, Write},
    net::SocketAddr,
    process::Command,
    sync::Mutex,
};
use once_cell::sync::Lazy;
use warp::{
    ws::{Message, WebSocket},
    Filter,
};
use futures_util::{StreamExt, SinkExt};
use tokio::time::{sleep, Duration};

// 顧客のIDを指定してサブドメインをDBServerに送信
pub fn insert_subdomain_to_db(customer_id: usize, subdomain: &str) -> Result<()> {
    let insert_query = format!(
        "UPDATE customer_data.customer_info SET subdomain = '{}' WHERE customer_id = {};",
        subdomain, customer_id
    );
    println!("Inserting subdomain into DB: {}", insert_query);
    Command::new("cqlsh")
        .arg("10.0.10.40")
        .arg("-e")
        .arg(insert_query)
        .output()?;
    Ok(())
}

// 顧客のIDを指定してそれに紐付いた仮想IPアドレスを取得
pub fn get_virtual_ips(customer_id: usize) -> Result<(String, String)> {
    let query = format!(
        "SELECT vpn_ip_client, vpn_ip_server FROM customer_data.customer_info WHERE customer_id = {};",
        customer_id
    );
    println!("Executing query: {}", query);
    let output = Command::new("cqlsh")
        .arg("10.0.10.40")
        .arg("-e")
        .arg(query)
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    println!("Raw output:\n{}", output_str);

    let mut client_ip = None;
    let mut server_ip = None;

    // DBServerからの出力からIPアドレス部分のみを抽出
    for line in output_str.lines() {
        if line.trim().is_empty() || line.starts_with("WARNING") || line.contains("vpn_ip_client") {
            continue;
        }
        
        let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            client_ip = Some(parts[0].to_string());
            server_ip = Some(parts[1].to_string());
            break;
        }
    }
    match (client_ip, server_ip) {
        (Some(c_ip), Some(s_ip)) => {
            println!("Retrieved IPs: Client IP: {}, Server IP: {}", c_ip, s_ip);
            Ok((c_ip, s_ip))
        }
        _ => {
            eprintln!("Error: Could not parse the expected number of fields.");
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "IP addresses not found"))
        }
    }
}

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

pub async fn handle_socket(ws: WebSocket) {
    let (mut tx, mut rx) = ws.split();
    while let Some(result) = rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    let text = msg.to_str().unwrap();
                    println!("ProxyServer received a message: {}", text);

                    let customer_id: usize = text.parse().unwrap_or(0);
                    if customer_id > 0 {
                        println!("Receive instructions from APIServer");

                        // IPアドレスを取得
                        if let Some((client_ip, server_ip)) = wait_for_virtual_ips(customer_id).await {
                            println!("Retrieved IPs: Client IP: {}, Server IP: {}", client_ip, server_ip);

                            // サブドメインを生成して登録
                            match generate_and_add_subdomain(&client_ip) {
                                Ok(subdomain) => {
                                    // サブドメインをDBに保存
                                    match insert_subdomain_to_db(customer_id, &subdomain) {
                                        Ok(_) => {
                                            println!("Successfully inserted subdomain into DB: {}", subdomain);

                                            let response = format!(
                                                "Subdomain: {}, Client IP: {}, Server IP: {}",
                                                subdomain, client_ip, server_ip
                                            );
                                            tx.send(Message::text(response)).await.unwrap();
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to insert subdomain into DB: {}", e);
                                            tx.send(Message::text("Error inserting subdomain to DB")).await.unwrap();
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to generate and add subdomain: {}", e);
                                    tx.send(Message::text("Error generating and adding subdomain")).await.unwrap();
                                }
                            }
                        } else {
                            eprintln!("Failed to retrieve IPs from DB for Customer ID: {}", customer_id);
                            tx.send(Message::text("Error retrieving IPs from DB")).await.unwrap();
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }

    if let Err(e) = tx.close().await {
        eprintln!("Failed to close WebSocket connection: {}", e);
    }
}

async fn wait_for_virtual_ips(customer_id: usize) -> Option<(String, String)> {
    let max_retries = 10; // 最大リトライ回数
    let retry_interval = Duration::from_millis(500); // リトライ間隔

    for attempt in 1..=max_retries {
        if let Ok((client_ip, server_ip)) = get_virtual_ips(customer_id) {
            if client_ip != "null" && server_ip != "null" {
                println!("DB Update verified for ID: {}. Retrieved IPs: Client IP: {}, Server IP: {}", customer_id, client_ip, server_ip);
                return Some((client_ip, server_ip));
            } else {
                println!("DB Update check attempt {} failed for ID: {}", attempt, customer_id);
            }
        }

        // リトライ間隔の待機
        sleep(retry_interval).await;
    }

    println!("DB Update confirmation failed after {} attempts for ID: {}", max_retries, customer_id);
    None
}

#[tokio::main]
async fn main() {
    // WebSocketサーバーを非同期タスクで起動
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handle_socket)
        });

    let addr = "0.0.0.0:8100".parse::<SocketAddr>().expect("Unable to parse socket address");
    println!("ProxyServer is running on {}", addr);

    tokio::spawn(async move {
        warp::serve(ws_route).run(addr).await;
    });

    // メイン関数が終了しないように待機
    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
}

