use nutype::nutype;

#[nutype(derive(
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Debug,
    Serialize,
    Deserialize,
    AsRef,
    From
))]
pub(crate) struct DataApiKey(String);

impl Default for DataApiKey {
    fn default() -> Self {
        let key = std::env::var("DATA_API_KEY")
            .expect("DATA_API_KEY must be set as an environment variable");
        DataApiKey::new(key)
    }
}
