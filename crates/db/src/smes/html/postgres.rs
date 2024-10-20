use crate::schema::smes::html::dsl;
use crate::smes::HtmlDb;
use crate::{schema, DbError, PostgresDb};
use diesel::prelude::*;
use diesel::upsert::excluded;
use hashbrown::HashSet;
use model::{table, ModelError};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedReceiver;

impl HtmlDb for PostgresDb {
    async fn select_html(
        &mut self,
        company_id: &str,
    ) -> Result<Option<crate::model::smes::Html>, DbError> {
        Ok(dsl::html
            .filter(dsl::company_id.eq(company_id))
            .first(&mut self.conn)
            .optional()?)
    }

    async fn select_htmls(&mut self) -> Result<Vec<crate::model::smes::Html>, DbError> {
        Ok(dsl::html.load(&mut self.conn)?)
    }

    async fn select_html_ids(&mut self) -> Result<HashSet<String>, DbError> {
        Ok(dsl::html
            .select(dsl::company_id)
            .load(&mut self.conn)?
            .into_iter()
            .collect())
    }

    async fn insert_html_channel(
        &mut self,
        mut htmls: UnboundedReceiver<crate::model::smes::NewHtml>,
    ) -> Result<(), DbError> {
        let query = diesel::insert_into(schema::smes::html::table);

        while let Some(html) = htmls.recv().await {
            query.values(&html).execute(&mut self.conn)?;
        }
        Ok(())
    }

    async fn upsert_html_channel(
        &mut self,
        mut htmls: UnboundedReceiver<crate::model::smes::NewHtml>,
    ) -> Result<(), DbError> {
        let query = diesel::insert_into(schema::smes::html::table);

        while let Some(html) = htmls.recv().await {
            query
                .values(&html)
                .on_conflict(dsl::company_id)
                .do_update()
                .set((dsl::html_raw.eq(excluded(dsl::html_raw)),))
                .execute(&mut self.conn)?;
        }
        Ok(())
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
