use std::process::Command;
use std::io::Result;
use tracing::{info, error};

// 顧客のIDを指定してサブドメインをDBServerに送信
pub fn insert_subdomain_to_db(customer_id: usize, subdomain: &str) -> Result<()> {
    let insert_query = format!(
        "UPDATE customer_data.customer_info SET subdomain = '{}' WHERE customer_id = {};",
        subdomain, customer_id
    );
    info!("Inserting subdomain into DB: {}", insert_query);
    let output = Command::new("cqlsh")
        .arg("10.0.10.40")
        .arg("-e")
        .arg(&insert_query)
        .output()?;

    if !output.status.success() {
        error!("Failed to insert subdomain into DB for customer_id: {}. Stderr: {:?}", customer_id, String::from_utf8_lossy(&output.stderr));
    } else {
        info!("Successfully inserted subdomain '{}' into DB for customer_id: {}", subdomain, customer_id);
    }

    Ok(())
}

// 顧客のIDを指定してそれに紐付いた仮想IPアドレスを取得
pub fn get_virtual_ips(customer_id: usize) -> Result<(String, String)> {
    let query = format!(
        "SELECT vpn_ip_client, vpn_ip_server FROM customer_data.customer_info WHERE customer_id = {};",
        customer_id
    );
    info!("Executing query: {}", query);
    let output = Command::new("cqlsh")
        .arg("10.0.10.40")
        .arg("-e")
        .arg(&query)
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    info!("Raw output:\n{}", output_str);

    let mut client_ip = None;
    let mut server_ip = None;

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
            info!("Retrieved IPs: Client IP: {}, Server IP: {}", c_ip, s_ip);
            Ok((c_ip, s_ip))
        }
        _ => {
            error!("Error: Could not parse the expected number of fields for customer_id: {}", customer_id);
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "IP addresses not found"))
        }
    }
}
