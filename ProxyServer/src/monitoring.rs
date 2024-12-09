use std::thread;
use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;
use tracing::{info, error};

pub fn start_client() {
    loop {
        match TcpStream::connect("127.0.0.1:2000") {
            Ok(mut stream) => {
                let message = "Alive";
                if let Err(e) = stream.write_all(message.as_bytes()) {
                    error!("Failed to send message: {}", e);
                } else {
                    info!("Sent: {} for APIServer", message);
                }
            }
            Err(e) => {
                error!("Failed to connect: {}", e);
            }
        }
        thread::sleep(Duration::from_secs(5)); // 5秒ごとに送信
    }
}
