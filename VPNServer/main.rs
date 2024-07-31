use warp::Filter;
mod handlers;

#[tokio::main]
async fn main() {
    // トンネル生成指示受信
    let tunnel_route = warp::path("create_tunnel")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handlers::handle_create_tunnel);

    warp::serve(tunnel_route).run(([127, 0, 0, 1], 4040)).await;
}
