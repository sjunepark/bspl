#[macro_export]
macro_rules! impl_error {
    ($error:ident) => {
        #[cfg_attr(coverage_nightly, coverage(off))]
        impl std::error::Error for $error {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                self.source.as_ref().map(|e| e.as_ref())
            }
        }
    };
}
