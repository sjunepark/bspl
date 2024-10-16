use crate::db::LibsqlParams;
use crate::smes::HtmlDb;
use crate::{DbError, LibsqlDb};
use hashbrown::HashSet;
use libsql::named_params;
use model::{company, table};
use tokio::sync::mpsc::UnboundedReceiver;

impl HtmlDb for LibsqlDb {
    async fn get_html(&self, smes_id: &str) -> Result<Option<table::Html>, DbError> {
        let mut rows = self
            .connection
            .query(
                "SELECT * from smes_html WHERE smes_id = :smes_id",
                named_params! {
                    ":smes_id": smes_id,
                },
            )
            .await?;

        let row = rows.next().await?;
        let result = row
            .map(|r| libsql::de::from_row::<crate::smes::Html>(&r))
            .transpose()?
            .map(TryInto::<table::Html>::try_into)
            .transpose()?;

        if let Some(row) = rows.next().await? {
            panic!(
                "Multiple HTMLs found for smes_id: {:?}, got: {:?}",
                smes_id, row
            );
        }

        Ok(result)
    }

    #[tracing::instrument(skip(self))]
    async fn get_htmls(&self) -> Result<Vec<table::Html>, DbError> {
        self.get_all_from::<crate::smes::Html>("smes_html")
            .await?
            .into_iter()
            .map(TryInto::try_into)
            .collect()
    }

    async fn get_html_ids(&self) -> Result<HashSet<company::Id>, DbError> {
        self.get_all_ids_from("smes_html").await
    }

    #[tracing::instrument(skip(self, htmls))]
    /// Insert HTMLs into the HTML table.
    ///
    /// Each HTML will be inserted one by one.
    /// When an error occurs, the error will be logged in WARN level and the operation will continue.
    async fn insert_htmls(&self, mut htmls: UnboundedReceiver<table::Html>) -> Result<(), DbError> {
        let mut statement = self
            .connection
            .prepare(
                "INSERT into smes_html (smes_id, html)
VALUES (:smes_id, :html);",
            )
            .await
            .inspect_err(|_e| tracing::error!("Failed to prepare statement for inserting HTMLs"))?;

        while let Some(html) = htmls.recv().await {
            match statement
                .execute(Into::<crate::smes::Html>::into(html.clone()).params())
                .await
            {
                Ok(_number_of_rows) => {
                    tracing::info!(smes_id = ?html.smes_id, "Inserted HTML");
                }
                Err(error) => {
                    tracing::warn!(smes_id = ?html.smes_id, ?error, "Failed to insert HTML into the database");
                }
            };
            statement.reset()
        }

        Ok(())
    }

    /// Upsert HTMLs into the HTML table.
    ///
    /// Each HTML will be upserted one by one.
    /// When an error occurs, the error will be logged in WARN level and the operation will continue.
    ///
    /// When the upserting `smes_id` exists in the table, the `html` and `updated_date` will be updated.
    #[tracing::instrument(skip(self))]
    async fn upsert_htmls(&self, mut htmls: UnboundedReceiver<table::Html>) -> Result<(), DbError> {
        let mut statement = self
            .connection
            .prepare(
                "INSERT into smes_html (smes_id, html)
VALUES (:smes_id, :html)
ON CONFLICT (smes_id) DO UPDATE SET html         = excluded.html,
                                    updated_date = CURRENT_DATE;",
            )
            .await
            .inspect_err(|_e| tracing::error!("Failed to prepare statement for upserting HTMLs"))?;

        while let Some(html) = htmls.recv().await {
            match statement
                .execute(Into::<crate::smes::Html>::into(html.clone()).params())
                .await
            {
                Ok(_number_of_rows) => {
                    tracing::trace!(smes_id = ?html.smes_id, "Upserted HTML");
                }
                Err(error) => {
                    tracing::warn!(?error, ?html, "Failed to upsert HTML into the database");
                }
            };
            statement.reset()
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::smes::HtmlDb;
    use crate::test_utils::{LibsqlTestContext, TestContext};
    use fake::{Fake, Faker};
    use model::table;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn get_html_should_work() {
        // region: Arrange
        tracing_setup::span!("test");

        let function_id = utils::function_id!();
        let ctx = LibsqlTestContext::new(&function_id).await;
        let db = ctx.db();

        let ids = (0..10_u64).map(|i| 1000000 + i).collect::<Vec<_>>();
        let htmls = ctx.populate_htmls(&ids).await;
        let html_to_get = htmls.clone().remove(0);
        let (tx, rx) = mpsc::unbounded_channel();
        for html in htmls {
            tx.send(html).expect("Failed to send HTML");
        }
        drop(tx);

        db.insert_htmls(rx).await.expect("Failed to insert HTMLs");
        // endregion: Arrange

        // region: Action
        let result = db
            .get_html(&html_to_get.smes_id)
            .await
            .expect("Failed to get HTML")
            .expect("No HTML found");
        // endregion: Action

        // region: Assert
        assert_eq!(result.smes_id, html_to_get.smes_id);
        assert_eq!(result.html, html_to_get.html);
        // endregion: Assert
    }

    #[tokio::test]
    async fn upsert_htmls_should_work() {
        // region: Arrange
        tracing_setup::span!("test");

        let function_id = utils::function_id!();
        let ctx = LibsqlTestContext::new(&function_id).await;
        let db = ctx.db();

        // Set up basic HTMLs
        let ids = (0..10_u64).map(|i| 1000000 + i).collect::<Vec<_>>();
        let htmls = ctx.populate_htmls(&ids).await;

        // Create HTMLs to upsert: from existing HTMLs
        const UPDATED_HTML_CONTENT: &str =
            "<html><body><h2>유동자산</h2><p>Updated</p></body></html>";
        let mut updated_htmls = htmls
            .into_iter()
            .map(|h| table::Html {
                html: UPDATED_HTML_CONTENT
                    .try_into()
                    .expect("failed to create dummy html"),
                ..h
            })
            .collect::<Vec<_>>();

        // Add a new HTML to see that this HTML was properly inserted
        let mut new_html = Faker.fake::<table::Html>();
        const NEW_HTML_ID: &str = "2000000";
        new_html.smes_id = NEW_HTML_ID
            .try_into()
            .expect("failed to create dummy smes_id");
        let new_html_content = new_html.html.clone();
        updated_htmls.push(new_html);

        // Remove an HTML to check that this HTML was not updated
        let removed_html = updated_htmls.pop().unwrap();
        let removed_html_id = removed_html.smes_id.as_str();
        // endregion: Arrange

        // region: Action
        let (tx, rx) = mpsc::unbounded_channel();
        for html in updated_htmls {
            tx.send(html).unwrap();
        }
        drop(tx);

        db.upsert_htmls(rx).await.expect("Failed to upsert HTMLs");
        // endregion: Action

        // region: Assert
        let db_htmls = db.get_htmls().await.expect("Failed to get HTMLs");

        for html in &db_htmls {
            match html.smes_id.as_str() {
                NEW_HTML_ID => {
                    assert_eq!(html.html, new_html_content);
                }
                id if id == removed_html_id => {
                    // Not upserted HTML content should not change
                    assert_eq!(html.html, removed_html.html);
                }
                _ => {
                    assert_eq!(
                        html.html,
                        UPDATED_HTML_CONTENT
                            .try_into()
                            .expect("failed to create dummy html")
                    );
                }
            }
        }
        // endregion: Assert
    }
}
