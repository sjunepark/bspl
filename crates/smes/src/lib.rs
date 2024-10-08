#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod api;
mod error;
mod utils;

pub(crate) use api::{Company, Html};

pub use api::{get_bspl_htmls, BsplApi, ListApi, ListPayload, ListPayloadBuilder, ListResponse};
pub use error::SmesError;
