fn main() -> Result<(), dns_over_https::BoxError> {
    dotenvy::dotenv().ok();

    let args = dns_over_https::Args::parse();

    #[cfg(target_os = "windows")]
    if args.service {
        dns_over_https::start_service()?;
        return Ok(());
    }

    let level = format!("{}={:?}", module_path!(), args.verbosity);
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level)).init();

    let join = ctrlc2::set_handler(|| {
        log::info!("Ctrl-C received, exiting...");
        unsafe { dns_over_https::dns_over_https_stop() };
        true
    })?;

    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    rt.block_on(async {
        dns_over_https::main_loop(&args).await?;
        Ok::<(), dns_over_https::Error>(())
    })?;

    join.join().expect("Couldn't join on the associated thread");

    Ok(())
}
