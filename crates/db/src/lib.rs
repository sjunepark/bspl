#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod db;
mod error;
pub mod smes;
mod test_utils;

pub use db::LibsqlDb;
pub use error::DbError;
pub use smes::{Company, Html};
