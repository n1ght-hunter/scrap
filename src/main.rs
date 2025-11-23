fn setup_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_ansi(true)
        .init();
}

fn main() {
    setup_logging();

    scrap_driver::run_complier(std::env::args_os());
}
