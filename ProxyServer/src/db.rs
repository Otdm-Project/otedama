use std::process::Command;
use std::io::Result;

// 顧客のIDを指定してサブドメインをDBServerに送信
pub fn insert_subdomain_to_db(customer_id: usize, subdomain: &str) -> Result<()> {
    let insert_query = format!(
        "UPDATE customer_data.customer_info SET subdomain = '{}' WHERE customer_id = {};",
        subdomain, customer_id
    );
    println!("Inserting subdomain into DB: {}", insert_query);
    Command::new("cqlsh")
        .arg("<DBServerのIPアドレス>")
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
        .arg("<DBServerのIPアドレス>")
        .arg("-e")
        .arg(query)
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    println!("Raw output:\n{}", output_str);

    let mut client_ip = None;
    let mut server_ip = None;

    //DBServerより取得した表形式の出力からIPアドレスの部分のみを抽出
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
