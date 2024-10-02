use tracing_subscriber::EnvFilter;

pub fn subscribe() {
    let subscriber = tracing_subscriber::fmt()
        .with_test_writer()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .inspect_err(|e| {
            tracing::trace!("Failed to set subscriber: {:?}", e);
        })
        .ok();
}
