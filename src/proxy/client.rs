use anyhow::Result;
use tokio::{
    io,
    net::{TcpListener, TcpStream},
    try_join,
};

pub async fn start_listener(listen_addr: &str, forward_addr: &str) -> Result<()> {
    let listener = TcpListener::bind(listen_addr).await?;
    let (client_stream, _) = listener.accept().await?;

    if let Err(e) = forward_to_server(client_stream, forward_addr).await {
        println!("Failed to forward: {}", e);
    }

    Ok(())
}

async fn forward_to_server(mut client_stream: TcpStream, server_addr: &str) -> Result<()> {
    let mut server_stream = TcpStream::connect(server_addr).await?;

    let (mut rc, mut wc) = client_stream.split();
    let (mut rs, mut ws) = server_stream.split();

    let client_to_server = io::copy(&mut rc, &mut ws);
    let server_to_client = io::copy(&mut rs, &mut wc);

    try_join!(client_to_server, server_to_client)?;

    Ok(())
}
