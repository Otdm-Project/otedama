use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

// クライアント側: "Alive"メッセージを5秒ごとに送信
fn start_client() {
    loop {
        match TcpStream::connect("127.0.0.1:2000") {
            Ok(mut stream) => {
                let message = "Alive";
                stream.write_all(message.as_bytes()).unwrap();
                println!("Sent: {}", message);
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        }
        thread::sleep(Duration::from_secs(5)); // 5秒ごとに送信
    }
}

// サーバ側: メッセージを受信
fn start_server() {
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

fn main() {
    // サーバを別スレッドで起動
    let server_handle = thread::spawn(|| {
        start_server();
    });

    // クライアントを起動
    let client_handle = thread::spawn(|| {
        start_client();
    });

    // 両スレッドを終了まで待機
    server_handle.join().unwrap();
    client_handle.join().unwrap();
}
