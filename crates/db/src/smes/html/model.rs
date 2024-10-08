use crate::db::Params;
use crate::error::ConversionError;
use crate::DbError;
use chrono::NaiveDate;
use libsql::params::IntoParams;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Html {
    pub smes_id: String,
    pub html: Vec<u8>,
    pub created_date: Option<NaiveDate>,
    pub updated_date: Option<NaiveDate>,
}

impl From<model::table::Html> for Html {
    fn from(value: model::table::Html) -> Self {
        Self {
            smes_id: value.smes_id.to_string(),
            html: value.html.to_vec(),
            created_date: value.created_date,
            updated_date: value.updated_date,
        }
    }
}

impl TryFrom<Html> for model::table::Html {
    type Error = DbError;

    fn try_from(value: Html) -> Result<Self, Self::Error> {
        Ok(model::table::Html {
            smes_id: value.smes_id.try_into().map_err(ConversionError::new)?,
            html: value.html.into(),
            created_date: value.created_date,
            updated_date: value.updated_date,
        })
    }
}

impl Params for Html {
    fn params(&self) -> impl IntoParams {
        libsql::named_params! {
            ":smes_id": self.smes_id.as_str(),
            ":html": self.html.as_slice(),
        }
    }
}
