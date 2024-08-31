mod handler;
mod subdomain;
mod db;

use warp::Filter;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            ws.on_upgrade(handler::handle_socket)
        });

    let addr = "0.0.0.0:8100".parse::<SocketAddr>().expect("Unable to parse socket address");
    println!("ProxyServer is running on {}", addr);

    warp::serve(ws_route).run(addr).await;
}
