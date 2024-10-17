use crate::DbError;
use hashbrown::HashSet;
use model::{company, table};
use std::future::Future;
use tokio::sync::mpsc::UnboundedReceiver;

pub trait HtmlDb {
    fn select_html(
        &self,
        smes_id: &str,
    ) -> impl Future<Output = Result<Option<table::Html>, DbError>>;
    fn select_htmls(&self) -> impl Future<Output = Result<Vec<table::Html>, DbError>>;
    fn select_html_ids(&self) -> impl Future<Output = Result<HashSet<company::Id>, DbError>>;
    fn insert_html_channel(
        &self,
        htmls: UnboundedReceiver<table::Html>,
    ) -> impl Future<Output = Result<(), DbError>>;
    fn upsert_html_channel(
        &self,
        htmls: UnboundedReceiver<table::Html>,
    ) -> impl Future<Output = Result<(), DbError>>;
}
