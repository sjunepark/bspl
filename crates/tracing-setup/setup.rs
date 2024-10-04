use tracing_subscriber::EnvFilter;

pub fn subscribe() {
    let test_log = std::env::var("TEST_LOG")
        .unwrap_or("false".to_string())
        .parse::<bool>()
        .expect("Failed to parse TEST_LOG to bool");

    match test_log {
        false => {
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
        true => {
            let subscriber = tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::from_default_env())
                .pretty()
                .finish();

            tracing::subscriber::set_global_default(subscriber)
                .inspect_err(|e| {
                    tracing::trace!("Failed to set subscriber: {:?}", e);
                })
                .ok();
        }
    }
}
