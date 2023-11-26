use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub async fn start_listener(listen_addr: &str, forward_addr: &'static str) -> Result<()> {
    let listener = TcpListener::bind(listen_addr).await?;
    while let Ok((inbound, _)) = listener.accept().await {
        tokio::spawn(forward_to_server(inbound, forward_addr));
    }

    Ok(())
}

async fn forward_to_server(mut client_stream: TcpStream, server_addr: &str) -> Result<()> {
    let mut server_stream = TcpStream::connect(server_addr).await?;

    let (mut rc, mut wc) = client_stream.split();
    let (mut rs, mut ws) = server_stream.split();

    let client_to_server = async {
        let mut buf = vec![0; 4096];
        loop {
            let n = rc.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            let content = &buf[0..n];
            ws.write_all(content).await?;
        }
        ws.shutdown().await
    };

    let server_to_client = async {
        let mut buf = vec![0; 4096];
        loop {
            let n = rs.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            let content = &buf[0..n];
            wc.write_all(content).await?;
        }
        wc.shutdown().await
    };

    tokio::try_join!(client_to_server, server_to_client)?;

    wc.shutdown().await?;
    ws.shutdown().await?;
    Ok(())
}
