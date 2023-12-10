pub mod crypto;
pub mod protocol;

use anyhow::Result;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use self::crypto::{decrypt, encrypt, identity};

#[derive(Debug, Clone, PartialEq)]
pub enum ProxyMode {
    Client,
    Server,
    Proxy,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChannelType {
    Receiving,
    Sending,
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
    proxy_mode: ProxyMode,
    channel_type: ChannelType,
    mut read_stream: impl tokio::io::AsyncRead + Unpin,
    mut write_stream: impl AsyncWrite + Unpin,
    process_data: CryptoFn,
) -> Result<()> {
    loop {
        let content = match (proxy_mode.clone(), channel_type.clone()) {
            (ProxyMode::Client, ChannelType::Sending)
            | (ProxyMode::Server, ChannelType::Receiving) => {
                protocol::custom_read_protocol(&mut read_stream, process_data).await?
            }
            _ => {
                let content =
                    protocol::standard_read_protocol(&mut read_stream, process_data).await?;
                if content.is_empty() {
                    // Tell server that client has no more data to send
                    let empty_byte: [u8; 2] = [0, 0];
                    write_stream.write_all(&empty_byte).await?;
                }

                content
            }
        };

        if content.is_empty() {
            write_stream.shutdown().await?;
            break;
        }

        match (proxy_mode.clone(), channel_type.clone()) {
            (ProxyMode::Client, ChannelType::Receiving)
            | (ProxyMode::Server, ChannelType::Sending) => {
                protocol::custom_write_protocol(content, &mut write_stream).await?
            }
            _ => protocol::standard_write_protocol(content, &mut write_stream).await?,
        }
    }

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

    let client_to_server = transfer_data(
        proxy_mode.clone(),
        ChannelType::Receiving,
        rc,
        ws,
        inbound_fn,
    );
    let server_to_client = transfer_data(
        proxy_mode.clone(),
        ChannelType::Sending,
        rs,
        wc,
        outbound_fn,
    );

    // await and log each error
    tokio::try_join!(client_to_server, server_to_client)?;

    Ok(())
}
