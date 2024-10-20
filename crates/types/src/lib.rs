//! # Model
//!
//! - This crate contains the domain models used in the project.
//! - They are usually implemented as newtypes.
//! - They will be validated during construction, usually with the `try_new` or `try_from` method.
//! - When no failures are expected, the `new` or `from` method can be used.

pub mod company;
mod error;
mod macros;
mod statics;
mod utils;

pub use error::TypeError;
