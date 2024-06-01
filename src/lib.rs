pub use crate::args::{ArgVerbosity, Args};
pub use crate::error::{BoxError, Error, Result};
use crate::udp_server::UdpServer;
#[cfg(target_os = "windows")]
pub use crate::windows::start_service;
use futures::stream::StreamExt;
use std::ffi::{c_char, c_int, CStr};
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio_util::sync::CancellationToken;

mod args;
mod dump_logger;
mod error;
mod udp_server;
mod upstream;
mod windows;

/// # Safety
/// Run the proxy server of DNS-over-HTTPS, this function will block the current thread.
/// The parameters are:
/// - `bind1`: The first bind address (IPv4 or IPv6) for the proxy server, e.g. `0.0.0.0:53`.
/// - `bind2`: The second bind address (IPv4 or IPv6) for the proxy server. e.g. `[::]:53`.
/// - `upstream_url`: The URL of the upstream service, e.g. `https://1.1.1.1/dns-query`.
/// - `verbosity`: The verbosity level of the log, e.g. `info`.
#[no_mangle]
pub unsafe extern "C" fn dns_over_https_run(
    bind1: *const c_char,
    bind2: *const c_char,
    upstream_url: *const c_char,
    verbosity: ArgVerbosity,
) -> c_int {
    let main_1_loop = async move {
        log::set_max_level(verbosity.into());

        #[cfg(target_os = "android")]
        {
            let filter_str = &format!("off,dns_over_https={verbosity}");
            let filter = android_logger::FilterBuilder::new().parse(filter_str).build();
            android_logger::init_once(
                android_logger::Config::default()
                    .with_tag("dns_over_https")
                    .with_max_level(log::LevelFilter::Trace)
                    .with_filter(filter),
            );
        }

        #[cfg(not(target_os = "android"))]
        if let Err(err) = log::set_boxed_logger(Box::<crate::dump_logger::DumpLogger>::default()) {
            log::warn!("set logger error: {}", err);
        }

        log::info!("Starting dns_over_https...");

        let mut args = Args::default();
        if !bind1.is_null() {
            args.bind(CStr::from_ptr(bind1).to_str()?.parse::<SocketAddr>()?);
        }
        if !bind2.is_null() {
            args.bind(CStr::from_ptr(bind2).to_str()?.parse::<SocketAddr>()?);
        }
        if args.bind.is_empty() {
            args.bind("127.0.0.1:53".parse::<SocketAddr>()?);
            args.bind("[::1]:53".parse::<SocketAddr>()?);
        }
        let url = if upstream_url.is_null() {
            "https://1.1.1.1/dns-query".to_string()
        } else {
            CStr::from_ptr(upstream_url).to_str()?.to_string()
        };
        args.upstream_url(url);
        args.verbosity(verbosity);
        if let Err(err) = main_loop(&args).await {
            log::error!("main loop error: {}", err);
            return Err(err);
        }
        Ok(())
    };

    let exit_code = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Err(_e) => -3,
        Ok(rt) => match rt.block_on(main_1_loop) {
            Ok(_) => 0,
            Err(_e) => -4,
        },
    };

    exit_code
}

static SHUTTING_DOWN_TOKEN: std::sync::Mutex<Option<CancellationToken>> = std::sync::Mutex::new(None);

pub async fn main_loop(args: &Args) -> Result<()> {
    let shutdown_token = CancellationToken::new();
    if let Ok(mut lock) = SHUTTING_DOWN_TOKEN.lock() {
        if lock.is_some() {
            return Err("dns-over-https already started".into());
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
pub unsafe extern "C" fn dns_over_https_stop() -> std::ffi::c_int {
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
