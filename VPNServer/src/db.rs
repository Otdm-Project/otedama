use std::process::Command;
use std::io::Result;

// DBServerから顧客環境の公開鍵を取得
pub fn get_public_key(customer_id: usize) -> Result<String> {
    let query = format!("SELECT client_public_key FROM customer_data.customer_info WHERE customer_id = {};", customer_id);

    println!("Executing query: {}", query);

    let output = Command::new("cqlsh")
        .arg("10.0.10.40")
        .arg("-e")
        .arg(query)
        .output()?;

    let public_key_output = String::from_utf8_lossy(&output.stdout).to_string();

    // 出力の行を取得
    let mut lines = public_key_output.lines();

    // データ行まで進むためにヘッダーと区切り線をスキップ
    while let Some(line) = lines.next() {
        if line.trim().starts_with('-') {
            break; // 区切り線に到達したら次からデータ行
        }
    }

    // 次の行がデータ行であることを期待して取得
    if let Some(public_key_line) = lines.next() {
        let public_key = public_key_line.trim().to_string();
        println!("Extracted public key: {}", public_key);
        Ok(public_key)
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Public key not found"))
    }
}

// DBServerにPeer設定用の情報を送信
pub fn insert_tunnel_data(customer_id: usize, server_public_key: &str, client_ip: &str, server_ip: &str) -> Result<()> {
    let insert_query = format!(
        "UPDATE customer_data.customer_info SET server_public_key = '{}', vpn_ip_client = '{}', vpn_ip_server = '{}' WHERE customer_id = {};",
        server_public_key, client_ip, server_ip, customer_id
    );

    println!("Executing insert query: {}", insert_query);

    Command::new("cqlsh")
        .arg("10.0.10.40")
        .arg("-e")
        .arg(insert_query)
        .output()?;

    Ok(())
}

