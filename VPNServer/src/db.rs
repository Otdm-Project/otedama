use std::process::Command;
use std::io;

pub fn get_public_key(customer_id: usize) -> io::Result<String> {
    let query = format!("SELECT client_public_key FROM customer_data.customer_info WHERE customer_id = {};", customer_id);

    println!("Executing query: {}", query);

    let output = Command::new("cqlsh")
        .arg("<DBServerのグローバルIPアドレス>")
        .arg("-e")
        .arg(query)
        .output()?;

    let public_key_output = String::from_utf8_lossy(&output.stdout).to_string();


    // 公開鍵が取得できたかチェック
    let lines: Vec<&str> = public_key_output.lines().collect();
    if lines.len() > 5 && lines[4].trim().len() > 0 {
        let public_key = lines[4].trim().to_string();  // 5行目にデータがあると想定
        println!("Extracted public key: {}", public_key);
        Ok(public_key)
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, "Public key not found"))
    }
}
