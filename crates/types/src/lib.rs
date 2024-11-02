//! # Types
//!
//! - This crate contains the domain models used in the project.
//! - They are usually implemented as newtypes.
//! - They will be validated during construction, usually with the `try_new` method.
//! - When no failures are expected, the `new` method can be used.

mod base;
pub mod company;
pub mod date;
mod error;
pub mod filing;

pub use date::YYYYMMDD;
pub use error::TypeError;
