#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod db;
mod error;
pub mod smes;
mod test_utils;

pub(crate) use smes::{Company, Html};

pub use db::LibsqlDb;
pub use error::DbError;
