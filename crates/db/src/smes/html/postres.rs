use crate::smes::HtmlDb;
use crate::{DbError, PostgresDb};
use hashbrown::HashSet;
use model::company::Id;
use model::table::Html;
use tokio::sync::mpsc::UnboundedReceiver;

impl HtmlDb for PostgresDb {
    async fn get_html(&self, smes_id: &str) -> Result<Option<Html>, DbError> {
        todo!()
    }

    async fn get_htmls(&self) -> Result<Vec<Html>, DbError> {
        todo!()
    }

    async fn get_html_ids(&self) -> Result<HashSet<Id>, DbError> {
        todo!()
    }

    async fn insert_htmls(&self, htmls: UnboundedReceiver<Html>) -> Result<(), DbError> {
        todo!()
    }

    async fn upsert_htmls(&self, htmls: UnboundedReceiver<Html>) -> Result<(), DbError> {
        todo!()
    }
}
