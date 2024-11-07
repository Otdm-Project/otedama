use std::io::Write;
use std::net::TcpStream; 
use std::time::Duration;

use crate::thread;

// クライアント側: "Alive"メッセージを5秒ごとに送信
pub fn start_client() {
    loop {
        match TcpStream::connect("127.0.0.1:2000") {
            Ok(mut stream) => {
                let message = "Alive";
                stream.write_all(message.as_bytes()).unwrap();
                println!("Sent: {} for APIServer ", message);
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        }
        thread::sleep(Duration::from_secs(5)); // 5秒ごとに送信
    }
}


