// main.rs
mod websocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    websocket::start_websocket_server().await?;
    Ok(())
}

