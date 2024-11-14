use crate::schema::smes::html::dsl;
use crate::{DbError, PostgresDb};
use diesel::upsert::excluded;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use hashbrown::HashSet;
use std::future::Future;
use tokio::sync::mpsc::UnboundedReceiver;
use types::company;

impl HtmlDb for PostgresDb {
    async fn select_html(
        &mut self,
        smes_id: &str,
    ) -> Result<Option<crate::model::smes::Html>, DbError> {
        Ok(dsl::html
            .filter(dsl::smes_id.eq(smes_id))
            .first(&mut self.diesel_conn)
            .optional()?)
    }

    async fn select_htmls(&mut self) -> Result<Vec<crate::model::smes::Html>, DbError> {
        Ok(dsl::html.load(&mut self.diesel_conn)?)
    }

    async fn select_html_ids(&mut self) -> Result<HashSet<company::SmesId>, DbError> {
        Ok(dsl::html
            .select(dsl::smes_id)
            .load(&mut self.diesel_conn)?
            .into_iter()
            .collect())
    }

    #[tracing::instrument(skip(self, htmls))]
    async fn insert_html_channel(
        &mut self,
        mut htmls: UnboundedReceiver<crate::model::smes::NewHtml>,
    ) -> Result<(), DbError> {
        let query = diesel::insert_into(dsl::html);

        while let Some(html) = htmls.recv().await {
            tracing::trace!(?html, "Inserting html");
            query.values(&html).execute(&mut self.diesel_conn)?;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self, htmls))]
    async fn upsert_html_channel(
        &mut self,
        mut htmls: UnboundedReceiver<crate::model::smes::NewHtml>,
    ) -> Result<(), DbError> {
        let query = diesel::insert_into(dsl::html);

        while let Some(html) = htmls.recv().await {
            tracing::trace!(?html, "Upserting html");
            query
                .values(&html)
                .on_conflict(dsl::smes_id)
                .do_update()
                .set((dsl::html_content.eq(excluded(dsl::html_content)),))
                .execute(&mut self.diesel_conn)?;
        }
        Ok(())
    }
}

pub trait HtmlDb {
    fn select_html(
        &mut self,
        smes_id: &str,
    ) -> impl Future<Output = Result<Option<crate::model::smes::Html>, DbError>>;
    fn select_htmls(
        &mut self,
    ) -> impl Future<Output = Result<Vec<crate::model::smes::Html>, DbError>>;
    fn select_html_ids(
        &mut self,
    ) -> impl Future<Output = Result<HashSet<company::SmesId>, DbError>>;
    fn insert_html_channel(
        &mut self,
        htmls: UnboundedReceiver<crate::model::smes::NewHtml>,
    ) -> impl Future<Output = Result<(), DbError>>;
    fn upsert_html_channel(
        &mut self,
        htmls: UnboundedReceiver<crate::model::smes::NewHtml>,
    ) -> impl Future<Output = Result<(), DbError>>;
}

#[cfg(test)]
mod tests {
    use crate::model::smes::NewHtml;
    use crate::smes::HtmlDb;
    use crate::test_utils::{PostgresTestContext, TestContext};
    use fake::Fake;

    #[tokio::test]
    async fn insert_and_get_htmls_should_work() {
        let function_id = utils::function_id!();
        let mut ctx = PostgresTestContext::new(&function_id).await;

        let ids = (0..10_u64).map(|i| 1000000 + i).collect::<Vec<_>>();
        let mut inserted_htmls = ctx.populate_htmls(&ids).await;

        let mut selected_htmls: Vec<_> = ctx
            .db()
            .select_htmls()
            .await
            .expect("Failed to get htmls")
            .into_iter()
            .map(NewHtml::from)
            .collect();

        inserted_htmls.sort_by_key(|c| c.smes_id.clone());
        selected_htmls.sort_by_key(|c| c.smes_id.clone());

        assert_eq!(inserted_htmls, selected_htmls,);
    }

    #[tokio::test]
    async fn upsert_htmls_should_work() {
        // region: Arrange
        tracing_setup::span!("test");

        let function_id = utils::function_id!();
        let mut ctx = PostgresTestContext::new(&function_id).await;

        // Set up basic htmls
        let ids = (0..10_u64).map(|i| 1000000 + i).collect::<Vec<_>>();
        let htmls = ctx.populate_htmls(&ids).await;

        // Create htmls to upsert: from existing htmls.
        const UPDATED_HTML_CONTENT: &str = "<p>유동자산</p>";
        let mut updated_htmls = htmls
            .iter()
            .map(|c| NewHtml {
                html_content: UPDATED_HTML_CONTENT
                    .try_into()
                    .expect("failed to create dummy html_content"),
                ..c.clone()
            })
            .collect::<Vec<_>>();

        // Add a new HTML to see that this HTML was properly updated
        let mut new_html = ().fake::<NewHtml>();
        const NEW_COMPANY_ID: &str = "2000000";
        new_html.smes_id = NEW_COMPANY_ID
            .try_into()
            .expect("failed to create dummy smes_id");
        let new_html_content = new_html.html_content.clone();
        updated_htmls.push(new_html);

        // Remove an HTML to check that this HTML was not updated
        let removed_html = updated_htmls.pop().unwrap();
        let removed_html_id = removed_html.smes_id;
        // endregion: Arrange

        // region: Action
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<NewHtml>();
        for html in updated_htmls {
            tx.send(html).expect("Failed to send html to channel");
        }
        drop(tx);

        let db = ctx.db();
        db.upsert_html_channel(rx)
            .await
            .inspect_err(|e| tracing::error!(?e, "Failed to upsert htmls"))
            .unwrap();
        // endregion: Action

        // region: Assert
        let db_htmls = db
            .select_htmls()
            .await
            .inspect_err(|e| tracing::error!(?e, "Failed to get htmls"))
            .unwrap();

        for html in &db_htmls {
            match html.smes_id.as_ref().as_str() {
                NEW_COMPANY_ID => {
                    assert_eq!(html.html_content, new_html_content);
                }
                id if id == removed_html_id.as_ref() => {
                    // Not upserted HTML name should not change
                    assert_eq!(html.html_content, removed_html.html_content);
                }
                _ => {
                    assert_eq!(html.html_content.as_ref(), UPDATED_HTML_CONTENT);
                }
            }
        }
        // endregion: Assert
    }
}
