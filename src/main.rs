use std::env;
pub mod proxy;

use anyhow::{bail, Result};
use proxy::start_proxy;

use crate::proxy::ProxyMode;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut is_client = false;
    let mut is_server = false;
    let mut is_proxy = false;
    let mut listen_address = None;
    let mut forward_address = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-l" | "--listen" => {
                i += 1;
                if i < args.len() {
                    listen_address = Some(args[i].clone());
                }
            }
            "-f" | "--forward" => {
                i += 1;
                if i < args.len() {
                    forward_address = Some(args[i].clone());
                }
            }
            "--client" => {
                is_client = true;
            }
            "--server" => {
                is_server = true;
            }
            "--proxy" => {
                is_proxy = true;
            }
            _ => {
                println!("Unknown option: {}", args[i]);
            }
        }
        i += 1;
    }

    let proxy_mode = match (is_client, is_server, is_proxy) {
        (true, false, false) => ProxyMode::Client,
        (false, true, false) => ProxyMode::Server,
        (false, false, true) => ProxyMode::Proxy,
        _ => {
            bail!("Exactly one of --client, --server, or --proxy must be specified");
        }
    };

    match env::var("AES_GCM_KEY") {
        Ok(_) => {}
        Err(_) => {
            if proxy_mode != ProxyMode::Proxy {
                bail!("AES_GCM_KEY must be specified in environment");
            }
        }
    };

    let listen_address = listen_address.map_or_else(
        || {
            bail!("Listen address not specified");
        },
        |s| Ok(s),
    )?;
    let forward_address = forward_address
        .map_or_else(
            || {
                bail!("Forward address not specified");
            },
            |s| Ok(s),
        )?
        .clone();

    println!("Starting proxy in {:?} mode", proxy_mode);
    println!(
        "Listening on {} and forwarding to {}",
        listen_address, forward_address
    );

    start_proxy(proxy_mode, listen_address, forward_address).await?;

    Ok(())
}
