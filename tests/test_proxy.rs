use std::time::Duration;

use tcp_proxy::proxy::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::sleep;

#[tokio::test]
async fn test_proxy_mode() {
    let mock_server_port = find_free_port().await;
    let mock_server_handle = tokio::spawn(setup_mock_server(mock_server_port));
    wait_for_proxy(mock_server_port).await;

    let client_port = find_free_port().await;
    let proxy_handle = tokio::spawn(start_proxy(
        ProxyMode::Proxy,
        String::from(format!("127.0.0.1:{}", client_port)),
        String::from(format!("127.0.0.1:{}", mock_server_port)),
    ));
    wait_for_proxy(client_port).await;

    let test_string = String::from("Hello, server!");
    assert_tcp_echo(client_port, test_string.clone()).await;

    mock_server_handle.abort();
    proxy_handle.abort();
}

#[tokio::test]
async fn test_client_server_mode() {
    let mock_server_port = find_free_port().await;
    std::env::set_var("AES_GCM_KEY", "01234567890123456789012345678901");
    let mock_server_handle = tokio::spawn(setup_mock_server(mock_server_port));
    wait_for_proxy(mock_server_port).await;

    let server_port = find_free_port().await;
    let server_handle = tokio::spawn(start_proxy(
        ProxyMode::Server,
        String::from(format!("127.0.0.1:{}", server_port)),
        String::from(format!("127.0.0.1:{}", mock_server_port)),
    ));
    wait_for_proxy(server_port).await;

    let client_port = find_free_port().await;
    let client_handle = tokio::spawn(start_proxy(
        ProxyMode::Client,
        String::from(format!("127.0.0.1:{}", client_port)),
        String::from(format!("127.0.0.1:{}", server_port)),
    ));
    wait_for_proxy(client_port).await;

    let test_string = String::from("Hello, server!");
    assert_tcp_echo(client_port, test_string.clone()).await;

    mock_server_handle.abort();
    client_handle.abort();
    server_handle.abort();
}

#[tokio::test]
async fn test_client_server_mode_long() {
    let mock_server_port = find_free_port().await;
    std::env::set_var("AES_GCM_KEY", "01234567890123456789012345678901");
    let mock_server_handle = tokio::spawn(setup_mock_server(mock_server_port));
    wait_for_proxy(mock_server_port).await;

    let server_port = find_free_port().await;
    let server_handle = tokio::spawn(start_proxy(
        ProxyMode::Server,
        String::from(format!("127.0.0.1:{}", server_port)),
        String::from(format!("127.0.0.1:{}", mock_server_port)),
    ));
    wait_for_proxy(server_port).await;

    let client_port = find_free_port().await;
    let client_handle = tokio::spawn(start_proxy(
        ProxyMode::Client,
        String::from(format!("127.0.0.1:{}", client_port)),
        String::from(format!("127.0.0.1:{}", server_port)),
    ));
    wait_for_proxy(client_port).await;

    let test_string = generate_long_string(60);
    assert_tcp_echo(client_port, test_string.clone()).await;

    mock_server_handle.abort();
    client_handle.abort();
    server_handle.abort();
}

async fn setup_mock_server(port: u16) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .unwrap();
    println!("Server listening on port {}", port);

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            let mut buf = vec![0; 1024 * 8];

            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Failed to read from socket: {}", e);
                        return;
                    }
                };

                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("Failed to write to socket: {}", e);
                    return;
                }
            }
        });
    }
}

async fn assert_tcp_echo(port: u16, request_string: String) {
    let mut client = TcpStream::connect(format!("127.0.0.1:{}", port))
        .await
        .unwrap();
    let (mut reader, mut writer) = client.split();
    writer.write_all(request_string.as_bytes()).await.unwrap();

    let mut buffer = [0; 1024 * 4];
    let mut total_bytes_read = 0;

    loop {
        match reader.read(&mut buffer).await {
            Ok(0) => {
                break;
            }
            Ok(n) => {
                total_bytes_read += n;
                if total_bytes_read >= request_string.as_bytes().len() {
                    break;
                }
            }
            Err(e) => {
                panic!("Failed to read from socket: {}", e);
            }
        }
    }

    let response_string = String::from_utf8_lossy(&buffer[..total_bytes_read]).to_string();
    assert_eq!(response_string, request_string);
}

async fn find_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    match listener.local_addr() {
        Ok(addr) => addr.port(),
        Err(e) => panic!("Failed to get local port: {}", e),
    }
}

async fn wait_for_proxy(port: u16) {
    let mut attempts = 0;
    let max_attempts = 10;
    let delay = Duration::from_millis(500);

    while attempts < max_attempts {
        if TcpStream::connect(format!("127.0.0.1:{}", port))
            .await
            .is_ok()
        {
            return;
        }
        sleep(delay).await;
        attempts += 1;
    }

    panic!("Proxy did not start within the expected time")
}

fn generate_long_string(length: usize) -> String {
    let mut long_string = String::with_capacity(length);

    for _ in 0..length {
        long_string.push('A');
    }

    long_string
}
