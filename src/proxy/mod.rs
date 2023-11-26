pub mod crypto;

use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use self::crypto::{decrypt, encrypt, identity};

#[derive(Debug, Clone, PartialEq)]
pub enum ProxyMode {
    Client,
    Server,
    Proxy,
}

pub async fn start_proxy(
    proxy_mode: ProxyMode,
    listen_addr: String,
    forward_addr: String,
) -> Result<()> {
    let listener = TcpListener::bind(listen_addr).await?;
    while let Ok((inbound, _)) = listener.accept().await {
        let forward_addr = forward_addr.clone();
        let proxy_mode = proxy_mode.clone();
        tokio::spawn(async move {
            if let Err(e) = forward_to_server(inbound, forward_addr, proxy_mode).await {
                eprintln!("Error in forward_to_server: {}", e);
            }
        });
    }

    Ok(())
}

async fn transfer_data(
    mut read_stream: impl tokio::io::AsyncRead + Unpin,
    mut write_stream: impl AsyncWrite + Unpin,
    process_data: CryptoFn,
) -> Result<()> {
    let mut buf = vec![0; 1_000_000];
    loop {
        let n = read_stream.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        let content = &buf[0..n];
        let processed_content = process_data(content.to_vec())?;
        write_stream.write_all(&processed_content).await?;
    }

    write_stream.shutdown().await?;
    Ok(())
}

type CryptoFn = fn(Vec<u8>) -> Result<Vec<u8>>;

async fn forward_to_server(
    mut client_stream: TcpStream,
    server_addr: String,
    proxy_mode: ProxyMode,
) -> Result<()> {
    let mut server_stream = TcpStream::connect(server_addr).await?;

    let (rc, wc) = client_stream.split();
    let (rs, ws) = server_stream.split();

    let (inbound_fn, outbound_fn): (CryptoFn, CryptoFn) = match proxy_mode {
        ProxyMode::Client => (encrypt, decrypt),
        ProxyMode::Server => (decrypt, encrypt),
        ProxyMode::Proxy => (identity, identity),
    };

    let client_to_server = transfer_data(rc, ws, inbound_fn);
    let server_to_client = transfer_data(rs, wc, outbound_fn);

    tokio::try_join!(client_to_server, server_to_client)?;

    Ok(())
}
