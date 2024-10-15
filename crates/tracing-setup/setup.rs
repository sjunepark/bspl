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

#[macro_export]
macro_rules! span {
    ($span_name:expr) => {
        tracing_setup::subscribe();

        let function_id = $crate::function_id!();
        let _span = tracing::info_span!($span_name, ?function_id).entered();
    };
}
