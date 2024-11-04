use monitoring;

fn main() {
    monitoring();
}

fn monitoring() {
    // サーバを別スレッドで起動
    let server_handle = std::thread::spawn(|| {
        println!("monitoring S start!");
        monitoring::start_server();
    });

    // クライアントを起動
    let client_handle = std::thread::spawn(|| {
        println!("monitoring C start!");
        monitoring::start_client();
    });

    // 両スレッドを終了まで待機
    server_handle.join().unwrap();
    client_handle.join().unwrap();
}