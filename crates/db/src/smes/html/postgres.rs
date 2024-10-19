use crate::smes::HtmlDb;
use crate::{DbError, PostgresDb};
use hashbrown::HashSet;
use model::company::Id;
use model::{company, table, ModelError};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedReceiver;

impl HtmlDb for PostgresDb {
    async fn select_html(&self, company_id: &str) -> Result<Option<table::Html>, DbError> {
        sqlx::query_as!(
            PostgresHtml,
            "SELECT * from smes.html WHERE company_id = $1",
            company_id
        )
        .fetch_optional(&self.pool)
        .await?
        .map(table::Html::try_from)
        .transpose()
    }

    async fn select_htmls(&self) -> Result<Vec<table::Html>, DbError> {
        sqlx::query_as!(PostgresHtml, "SELECT * from smes.html")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(table::Html::try_from)
            .collect()
    }

    async fn select_html_ids(&self) -> Result<HashSet<Id>, DbError> {
        sqlx::query!("SELECT company_id from smes.html")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|html| {
                company::Id::try_from(html.company_id)
                    .map_err(|e| DbError::from(ModelError::from(e)))
            })
            .collect()
    }

    async fn insert_html_channel(
        &self,
        _htmls: UnboundedReceiver<table::Html>,
    ) -> Result<(), DbError> {
        todo!()
    }

    async fn upsert_html_channel(
        &self,
        _htmls: UnboundedReceiver<table::Html>,
    ) -> Result<(), DbError> {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PostgresHtml {
    pub company_id: String,
    pub html_raw: String,
    pub created_at: Option<time::PrimitiveDateTime>,
    pub updated_at: Option<time::PrimitiveDateTime>,
}

impl From<table::Html> for PostgresHtml {
    fn from(value: table::Html) -> Self {
        Self {
            company_id: value.company_id.to_string(),
            html_raw: value.html.into(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl TryFrom<PostgresHtml> for table::Html {
    type Error = DbError;

    fn try_from(value: PostgresHtml) -> Result<Self, Self::Error> {
        Ok(table::Html {
            company_id: value.company_id.try_into().map_err(ModelError::from)?,
            html: value.html_raw.try_into().map_err(ModelError::from)?,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}
