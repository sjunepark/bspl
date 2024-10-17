use crate::smes::HtmlDb;
use crate::{DbError, PostgresDb};
use hashbrown::HashSet;
use model::company::Id;
use model::{company, table, ModelError};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedReceiver;

impl HtmlDb for PostgresDb {
    async fn select_html(&self, smes_id: &str) -> Result<Option<table::Html>, DbError> {
        sqlx::query_as!(
            PostgresHtml,
            "SELECT * from smes_html WHERE smes_id = $1",
            smes_id
        )
        .fetch_optional(&self.pool)
        .await?
        .map(table::Html::try_from)
        .transpose()
    }

    async fn select_htmls(&self) -> Result<Vec<table::Html>, DbError> {
        sqlx::query_as!(PostgresHtml, "SELECT * from smes_html")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(table::Html::try_from)
            .collect()
    }

    async fn select_html_ids(&self) -> Result<HashSet<Id>, DbError> {
        sqlx::query!("SELECT smes_id from smes_html")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|html| {
                company::Id::try_from(html.smes_id).map_err(|e| DbError::from(ModelError::from(e)))
            })
            .collect()
    }

    async fn insert_html_channel(
        &self,
        htmls: UnboundedReceiver<table::Html>,
    ) -> Result<(), DbError> {
        todo!()
    }

    async fn upsert_html_channel(
        &self,
        htmls: UnboundedReceiver<table::Html>,
    ) -> Result<(), DbError> {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PostgresHtml {
    pub smes_id: String,
    pub html: String,
    pub created_date: Option<time::Date>,
    pub updated_date: Option<time::Date>,
}

impl From<table::Html> for PostgresHtml {
    fn from(value: table::Html) -> Self {
        Self {
            smes_id: value.smes_id.to_string(),
            html: value.html.into(),
            created_date: value.created_date,
            updated_date: value.updated_date,
        }
    }
}

impl TryFrom<PostgresHtml> for table::Html {
    type Error = DbError;

    fn try_from(value: PostgresHtml) -> Result<Self, Self::Error> {
        Ok(table::Html {
            smes_id: value.smes_id.try_into().map_err(ModelError::from)?,
            html: value.html.try_into().map_err(ModelError::from)?,
            created_date: value.created_date,
            updated_date: value.updated_date,
        })
    }
}
