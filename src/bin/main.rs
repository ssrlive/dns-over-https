fn main() -> Result<(), dns_over_tls::BoxError> {
    dotenvy::dotenv().ok();

    let args = dns_over_tls::Args::parse();

    let level = format!("{}={:?}", module_path!(), args.verbosity);
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level)).init();

    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    rt.block_on(async {
        dns_over_tls::main_loop(&args).await?;
        Ok::<(), dns_over_tls::Error>(())
    })?;
    Ok(())
}
