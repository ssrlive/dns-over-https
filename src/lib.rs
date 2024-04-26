pub use crate::args::Args;
pub use crate::error::{BoxError, Error, Result};
use crate::udp_server::UdpServer;
#[cfg(target_os = "windows")]
pub use crate::windows::start_service;
use futures::stream::StreamExt;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio_util::sync::CancellationToken;

mod args;
mod error;
mod udp_server;
mod upstream;
mod windows;

static SHUTTING_DOWN_TOKEN: std::sync::Mutex<Option<CancellationToken>> = std::sync::Mutex::new(None);

pub async fn main_loop(args: &Args) -> Result<()> {
    let shutdown_token = CancellationToken::new();
    if let Ok(mut lock) = SHUTTING_DOWN_TOKEN.lock() {
        if lock.is_some() {
            return Err("dns-over-tls already started".into());
        }
        *lock = Some(shutdown_token.clone());
    }

    let mut tasks = Vec::new();

    for bind in args.bind.clone() {
        let shutdown_token = shutdown_token.clone();
        let args = args.clone();
        let task = tokio::spawn(async move { _main_loop(shutdown_token, bind, &args).await });
        tasks.push(task);
    }

    let results = futures::future::join_all(tasks).await;

    for result in results {
        if let Err(e) = result {
            log::error!("Error in main loop: {:?}", e);
        }
    }
    Ok(())
}

/// # Safety
///
/// Shutdown the proxy server.
#[no_mangle]
pub unsafe extern "C" fn dns_over_tls_stop() -> std::ffi::c_int {
    log::info!("Shutting down...");
    if let Ok(mut token) = SHUTTING_DOWN_TOKEN.lock() {
        if let Some(token) = token.take() {
            token.cancel();
        }
    }
    0
}

async fn _main_loop(quit: CancellationToken, bind: SocketAddr, args: &Args) -> Result<()> {
    log::info!("Listening for DNS requests on {}...", bind);

    let socket = UdpSocket::bind(bind).await?;

    let mut server = UdpServer::new(&socket);

    let client = reqwest::Client::new();
    let upstreams = args.upstreams(&client);

    loop {
        tokio::select! {
            _ = quit.cancelled() => {
                log::info!("Listener on {} is shutting down...", bind);
                break;
            }
            result = server.next() => {
                match result.ok_or("error during receiving request")? {
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
        }
    }
    Ok(())
}
