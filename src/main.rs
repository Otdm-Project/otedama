use warp::Filter;
mod handlers;

#[tokio::main]
async fn main() {
    // WebSocket待受
    let websocket_route = warp::path("ws")
        .and(warp::ws())
        .and_then(handlers::handle_websocket);

    // 公開鍵待受
    let public_key_route = warp::path("public_key")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handlers::handle_public_key);

    // ルートの定義
    let routes = websocket_route.or(public_key_route);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
