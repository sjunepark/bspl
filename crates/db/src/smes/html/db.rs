use crate::DbError;
use hashbrown::HashSet;
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
