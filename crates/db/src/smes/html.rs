use crate::schema::smes::html;
use crate::{schema, DbError, PostgresDb};

use diesel::upsert::excluded;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use hashbrown::HashSet;
use std::future::Future;
use tokio::sync::mpsc::UnboundedReceiver;
use types::company;

impl HtmlDb for PostgresDb {
    async fn select_html(
        &mut self,
        company_id: &str,
    ) -> Result<Option<crate::model::smes::Html>, DbError> {
        Ok(html::table
            .filter(html::company_id.eq(company_id))
            .first(&mut self.conn)
            .optional()?)
    }

    async fn select_htmls(&mut self) -> Result<Vec<crate::model::smes::Html>, DbError> {
        Ok(html::table.load(&mut self.conn)?)
    }

    async fn select_html_ids(&mut self) -> Result<HashSet<company::Id>, DbError> {
        Ok(html::table
            .select(html::company_id)
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
                .on_conflict(html::company_id)
                .do_update()
                .set((html::html_content.eq(excluded(html::html_content)),))
                .execute(&mut self.conn)?;
        }
        Ok(())
    }
}

pub trait HtmlDb {
    fn select_html(
        &mut self,
        company_id: &str,
    ) -> impl Future<Output = Result<Option<crate::model::smes::Html>, DbError>>;
    fn select_htmls(
        &mut self,
    ) -> impl Future<Output = Result<Vec<crate::model::smes::Html>, DbError>>;
    fn select_html_ids(&mut self) -> impl Future<Output = Result<HashSet<company::Id>, DbError>>;
    fn insert_html_channel(
        &mut self,
        htmls: UnboundedReceiver<crate::model::smes::NewHtml>,
    ) -> impl Future<Output = Result<(), DbError>>;
    fn upsert_html_channel(
        &mut self,
        htmls: UnboundedReceiver<crate::model::smes::NewHtml>,
    ) -> impl Future<Output = Result<(), DbError>>;
}
