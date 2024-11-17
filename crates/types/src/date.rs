use crate::TypeError;
use chrono::NaiveDate;
use derive_more::{AsRef, Display};
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
    derive_more::From,
    derive_more::Into,
    // serde
    Serialize,
    Deserialize,
    // diesel
    DieselNewType,
    // SeaORM
    sea_orm::DeriveValueType,
)]
pub struct YYYYMMDD(NaiveDate);

impl YYYYMMDD {
    pub fn new(date: NaiveDate) -> Self {
        Self(date)
    }
}

impl FromStr for YYYYMMDD {
    type Err = TypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let date = NaiveDate::parse_from_str(s, "%Y%m%d")?;
        Ok(YYYYMMDD::new(date))
    }
}

impl TryFrom<&str> for YYYYMMDD {
    type Error = TypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[cfg(any(test, feature = "fake"))]
impl fake::Dummy<fake::Faker> for YYYYMMDD {
    fn dummy_with_rng<R: fake::Rng + ?Sized>(_config: &fake::Faker, _rng: &mut R) -> Self {
        YYYYMMDD::new(NaiveDate::from_ymd_opt(2021, 1, 1).expect("invalid date passed"))
    }
}
