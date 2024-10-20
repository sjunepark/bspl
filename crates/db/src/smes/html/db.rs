use crate::DbError;
use hashbrown::HashSet;
use std::future::Future;
use tokio::sync::mpsc::UnboundedReceiver;
use types::company;

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
