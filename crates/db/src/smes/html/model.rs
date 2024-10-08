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

// This implementation is necessary to create fake `Html` structs for tests,
// such as `().fake::<Html>().`
#[cfg(test)]
mod test_impl {
    use super::*;
    use chrono::Utc;
    use chrono_tz::Asia;
    use fake::faker::number::raw::NumberWithFormat;
    use fake::locales::EN;
    use fake::{Dummy, Fake};
    use rand::Rng;

    impl<T> Dummy<T> for Html {
        fn dummy_with_rng<R: Rng + ?Sized>(_config: &T, _rng: &mut R) -> Self {
            let html: &str = "<<html><body>Dummy</body></html>";
            let now = Utc::now().with_timezone(&Asia::Seoul).date_naive();

            Html {
                smes_id: NumberWithFormat(EN, "^######").fake::<String>(),
                html: html.as_bytes().to_owned(),
                created_date: Some(now),
                updated_date: Some(now),
            }
        }
    }
}
