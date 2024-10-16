use crate::{DbError, PostgresDb};
use hashbrown::HashSet;
use model::company::Id;
use model::table::Html;
use model::{company, table};
use std::future::Future;
use tokio::sync::mpsc::UnboundedReceiver;

pub trait HtmlDb {
    fn get_html(&self, smes_id: &str)
        -> impl Future<Output = Result<Option<table::Html>, DbError>>;
    fn get_htmls(&self) -> impl Future<Output = Result<Vec<table::Html>, DbError>>;
    fn get_html_ids(&self) -> impl Future<Output = Result<HashSet<company::Id>, DbError>>;
    fn insert_htmls(
        &self,
        htmls: UnboundedReceiver<table::Html>,
    ) -> impl Future<Output = Result<(), DbError>>;
    fn upsert_htmls(
        &self,
        htmls: UnboundedReceiver<table::Html>,
    ) -> impl Future<Output = Result<(), DbError>>;
}

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
