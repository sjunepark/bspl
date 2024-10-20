pub mod api;
mod error;
mod parser;
mod utils;

pub(crate) use api::{Company, Html};
pub use parser::Table;

pub use api::{get_bspl_htmls, BsplApi, ListApi, ListPayload, ListPayloadBuilder, ListResponse};
pub use error::SmesError;
