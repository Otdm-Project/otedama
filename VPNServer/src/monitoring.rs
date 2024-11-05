use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::thread;

// クライアント側: "Alive"メッセージを5秒ごとに送信
pub fn start_client() {
    loop {
        match TcpStream::connect("127.0.0.1:2000") {
            Ok(mut stream) => {
                let message = "Alive";
                stream.write_all(message.as_bytes()).unwrap();
                println!("Sent: {} for VPNServer ", message);
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        }
        thread::sleep(Duration::from_secs(5)); // 5秒ごとに送信
    }
}

// サーバ側: メッセージを受信
pub fn start_server() {
    let listener = TcpListener::bind("127.0.0.1:2000").unwrap();
    println!("Server is running on port 2000");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 1024];
                match stream.read(&mut buffer) {
                    Ok(size) => {
                        if size > 0 {
                            let received = String::from_utf8_lossy(&buffer[..size]);
                            println!("Received: {}", received);
                        }
                    }
                    Err(e) => {
                        println!("Failed to receive data: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("Failed to accept connection: {}", e);
            }
        }
    }
}

