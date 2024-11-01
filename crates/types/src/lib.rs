//! # Types
//!
//! - This crate contains the domain models used in the project.
//! - They are usually implemented as newtypes.
//! - They will be validated during construction, usually with the `try_new` method.
//! - When no failures are expected, the `new` method can be used.

mod error;

mod base;
pub mod company;
mod dart_common;
pub mod filing;

pub use dart_common::DartDate;
pub use error::TypeError;
