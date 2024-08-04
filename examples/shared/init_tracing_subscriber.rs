pub fn init_tracing_subscriber() {
    use tracing::level_filters::LevelFilter;
    use tracing_error::ErrorLayer;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        // .with_max_level(tracing::Level::TRACE) // Set the maximum log level to TRACE
        .finish()
        .with(ErrorLayer::default());
    // dbg!(&subscriber);
    subscriber.init();
}
