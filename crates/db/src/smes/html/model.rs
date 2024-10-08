use crate::db::Params;
use crate::DbError;
use chrono::{NaiveDate, Utc};
use chrono_tz::Asia;
use libsql::params::IntoParams;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Validate, PartialEq)]
pub struct Html {
    #[validate(length(min = 7, max = 7))]
    pub smes_id: String,
    pub html: Vec<u8>,
    pub create_date: NaiveDate,
    pub update_date: NaiveDate,
}

impl TryFrom<smes::BsPl> for Html {
    type Error = DbError;

    fn try_from(value: smes::BsPl) -> Result<Self, Self::Error> {
        let now = Utc::now().with_timezone(&Asia::Seoul).date_naive();

        let html = Self {
            smes_id: value.vnia_sn.to_string(),
            html: value.html.as_bytes().to_owned(),
            create_date: now,
            update_date: now,
        };
        html.validate()?;
        Ok(html)
    }
}

impl Params for Html {
    fn params(&self) -> impl IntoParams {
        libsql::named_params! {
            ":smes_id": self.smes_id.as_str(),
            ":html": self.html.as_slice(),
            ":create_date": self.create_date.to_string(),
            ":update_date": self.update_date.to_string(),
        }
    }
}

// This implementation is necessary to create fake `Html` structs for tests,
// such as `().fake::<Html>().`
#[cfg(test)]
mod test_impl {
    use super::*;
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
                create_date: now,
                update_date: now,
            }
        }
    }
}
