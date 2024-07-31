use warp::Filter;
mod handlers;

#[tokio::main]
async fn main() {
    // サブドメイン生成指示
    let subdomain_route = warp::path("create_subdomain")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handlers::handle_create_subdomain);

    warp::serve(subdomain_route).run(([127, 0, 0, 1], 5050)).await;
}
