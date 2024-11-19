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

fn main() {

    // クライアントを起動
    let client_handle = thread::spawn(|| {
        start_client();
    });

    // 両スレッドを終了まで待機
    client_handle.join().unwrap();
}
