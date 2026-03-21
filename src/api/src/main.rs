use luce_api::server::start_server;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    start_server(addr, None).await
}
