use crate::TypeError;
use chrono::NaiveDate;
use derive_more::{AsRef, Display, From, Into};
use diesel_derive_newtype::DieselNewType;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// YYYYMMDD date
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
    // serde
    Serialize,
    Deserialize,
    // diesel
    DieselNewType,
)]
pub struct DartDate(NaiveDate);

impl DartDate {
    pub fn new(date: NaiveDate) -> Self {
        Self(date)
    }
}

impl FromStr for DartDate {
    type Err = TypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let date = NaiveDate::parse_from_str(s, "%Y%m%d")?;
        Ok(DartDate::new(date))
    }
}

/// 비고
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
    // diesel
    DieselNewType,
)]
pub struct Remark(String);

impl Remark {
    pub fn new(value: &str) -> Self {
        Self(value.to_string())
    }
}
