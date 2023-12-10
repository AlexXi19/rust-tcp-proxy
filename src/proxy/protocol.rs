use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::CryptoFn;

pub async fn standard_read_protocol(
    mut read_stream: impl tokio::io::AsyncRead + Unpin,
    process_data: CryptoFn,
) -> Result<Vec<u8>> {
    let mut buf = vec![0; 1024];
    let n = read_stream.read(&mut buf).await?;
    if n == 0 {
        return Ok(vec![]);
    }
    let content = &buf[0..n];
    process_data(content.to_vec())
}

pub async fn custom_read_protocol(
    mut read_stream: impl tokio::io::AsyncRead + Unpin,
    process_data: CryptoFn,
) -> Result<Vec<u8>> {
    let mut length_buf = [0u8; 2];
    read_stream.read_exact(&mut length_buf).await?;
    let length = u16::from_be_bytes(length_buf) as usize;

    if length == 0 {
        return Ok(vec![]);
    }

    let mut buf = vec![0u8; length];
    read_stream.read_exact(&mut buf).await?;

    let res = process_data(buf.to_vec());
    res
}

pub async fn standard_write_protocol(
    content: Vec<u8>,
    mut write_stream: impl AsyncWrite + Unpin,
) -> Result<()> {
    if content.len() == 0 {
        write_stream.shutdown().await?;
        return Ok(());
    }

    write_stream.write_all(&content).await.unwrap();

    Ok(())
}

pub async fn custom_write_protocol(
    content: Vec<u8>,
    mut write_stream: impl AsyncWrite + Unpin,
) -> Result<()> {
    let content_length: u16 = content.len() as u16;
    let content_length_array = content_length.to_be_bytes();
    let mut content_with_length_byte = vec![0u8; content.len() + 2];

    content_with_length_byte[0] = content_length_array[0];
    content_with_length_byte[1] = content_length_array[1];

    content_with_length_byte[2..].copy_from_slice(&content[..]);

    write_stream.write_all(&content_with_length_byte).await?;

    Ok(())
}
