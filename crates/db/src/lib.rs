mod db;
mod error;

pub(crate) mod schema;
pub(crate) mod test_utils;

pub mod dart;
pub mod model;
pub mod smes;

pub use db::{Db, PostgresDb};
pub use error::DbError;

pub(crate) const POSTGRES_MAX_PARAMETERS: usize = 65535;
