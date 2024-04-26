pub use crate::args::Args;
pub use crate::error::{BoxError, Error, Result};
use crate::udp_server::UdpServer;
use futures::stream::StreamExt;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

mod args;
mod error;
mod udp_server;
mod upstream;

pub async fn main_loop(args: &Args) -> Result<()> {
    for bind in args.bind.clone() {
        _main_loop(bind, args).await?;
    }
    Ok(())
}

async fn _main_loop(bind: SocketAddr, args: &Args) -> Result<()> {
    log::info!("Listening for DNS requests on {}...", bind);

    let socket = UdpSocket::bind(bind).await?;

    let mut server = UdpServer::new(&socket);

    let client = reqwest::Client::new();
    let upstreams = args.upstreams(&client);

    while let Some(request) = server.next().await {
        match request {
            Ok(request) => {
                for upstream in upstreams.iter() {
                    match upstream.send(&request).await {
                        Ok(response) => {
                            server.reply(&request, &response).await?;
                            break;
                        }
                        Err(e) => {
                            log::error!("error during sending request: {:?}", e);
                            continue;
                        }
                    }
                }
            }
            Err(e) => log::trace!("error during DNS request: {:?}", e),
        }
    }
    Ok(())
}
