use derive_more::{AsRef, Display, From, Into};
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    // derive_more
    AsRef,
    Display,
    From,
    Into,
    // serde
    Serialize,
    Deserialize,
)]
pub(crate) struct DataApiKey(String);

impl DataApiKey {
    pub(crate) fn new(key: &str) -> Self {
        Self(key.to_string())
    }
}

impl Default for DataApiKey {
    fn default() -> Self {
        let key = std::env::var("DATA_API_KEY")
            .expect("DATA_API_KEY must be set as an environment variable");
        DataApiKey::new(&key)
    }
}
