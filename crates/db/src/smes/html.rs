use crate::DbError;
use chrono::{NaiveDate, Utc};
use chrono_tz::Asia;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct Html {
    #[validate(length(min = 7, max = 7))]
    pub id: String,
    pub html: Vec<u8>,
    pub create_date: NaiveDate,
    pub update_date: NaiveDate,
}

impl TryFrom<smes::BsPl> for Html {
    type Error = DbError;

    fn try_from(value: smes::BsPl) -> Result<Self, Self::Error> {
        let now = Utc::now().with_timezone(&Asia::Seoul).date_naive();

        let html = Self {
            id: value.vnia_sn.to_string(),
            html: value.html.as_bytes().to_owned(),
            create_date: now,
            update_date: now,
        };
        html.validate()?;
        Ok(html)
    }
}
