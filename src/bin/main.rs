fn main() {
    dotenvy::dotenv().ok();

    let args = dns_over_tls::Args::parse();

    let level = format!("{}={:?}", module_path!(), args.verbosity);
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level)).init();

    dns_over_tls::main_loop(&args).unwrap();
}
