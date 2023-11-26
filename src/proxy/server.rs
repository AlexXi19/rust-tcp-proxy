use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

// WIP

pub async fn start_server(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(address).await?;

    loop {
        let (socket, client_address) = listener.accept().await?;
        println!("Accepted connection from: {:?}", client_address);

        tokio::spawn(handle_connection(socket));
    }
}

async fn handle_connection(mut socket: TcpStream) {
    let mut buffer = [0u8; 1024];

    loop {
        match socket.read(&mut buffer).await {
            Ok(0) => {
                break;
            }
            Ok(n) => {
                if let Err(e) = socket.write_all(&buffer[0..n]).await {
                    eprintln!("Failed to write to socket: {}", e);
                    return;
                }
            }
            Err(e) => {
                eprintln!("Failed to read from socket: {}", e);
                return;
            }
        }
    }
}
