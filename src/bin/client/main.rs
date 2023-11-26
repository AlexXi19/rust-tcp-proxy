use anyhow::Result;
use tcp_proxy::proxy::client::start_listener;

#[tokio::main]
async fn main() -> Result<()> {
    start_listener("127.0.0.1:7878", "127.0.0.1:8000").await
}
