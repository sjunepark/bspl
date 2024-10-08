use crate::db::Params;
use crate::{DbError, Html, LibsqlDb};
use libsql::named_params;
use tokio::sync::mpsc::UnboundedReceiver;

impl LibsqlDb {
    pub async fn get_html(&self, smes_id: &str) -> Result<Option<Html>, DbError> {
        let mut rows = self
            .connection
            .query(
                "SELECT * from smes_html WHERE smes_id = :smes_id",
                named_params! {
                    ":smes_id": smes_id,
                },
            )
            .await?;

        let result = if let Some(row) = rows.next().await? {
            let html = libsql::de::from_row::<Html>(&row)?;
            Ok(Some(html))
        } else {
            Ok(None)
        };

        if let Some(row) = rows.next().await? {
            panic!(
                "Multiple HTMLs found for smes_id: {:?}, got: {:?}",
                smes_id, row
            );
        }

        result
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_htmls(&self) -> Result<Vec<Html>, DbError> {
        self.get_all_from::<Html>("smes_html").await
    }

    #[tracing::instrument(skip(self))]
    /// Insert HTMLs into the HTML table.
    ///
    /// Each HTML will be inserted one by one.
    /// When an error occurs, the error will be logged in WARN level and the operation will continue.
    async fn insert_htmls(&self, mut htmls: UnboundedReceiver<Html>) -> Result<(), DbError> {
        let mut statement = self
            .connection
            .prepare(
                "INSERT into smes_html (smes_id, html, create_date, update_date)
VALUES (:smes_id, :html, :create_date, :update_date);",
            )
            .await
            .inspect_err(|_e| tracing::error!("Failed to prepare statement for inserting HTMLs"))?;

        while let Some(html) = htmls.recv().await {
            match statement.execute(html.params()).await {
                Ok(_number_of_rows) => {
                    tracing::trace!(smes_id = ?html.smes_id, "Inserted HTML");
                }
                Err(error) => {
                    tracing::warn!(?error, ?html, "Failed to insert HTML into the database");
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
    /// When the upserting `smes_id` exists in the table, the `html` and `update_date` will be updated.
    #[tracing::instrument(skip(self))]
    pub async fn upsert_htmls(&self, mut htmls: UnboundedReceiver<Html>) -> Result<(), DbError> {
        let mut statement = self
            .connection
            .prepare(
                "INSERT into smes_html (smes_id, html, create_date, update_date)
VALUES (:smes_id, :html, :create_date, :update_date)
ON CONFLICT (smes_id) DO UPDATE SET html        = excluded.html,
                                    update_date = excluded.update_date;",
            )
            .await
            .inspect_err(|_e| tracing::error!("Failed to prepare statement for upserting HTMLs"))?;

        while let Some(html) = htmls.recv().await {
            match statement.execute(html.params()).await {
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
    use crate::test_utils::{text_context, DbSource, TestContext};
    use crate::{Html, LibsqlDb};
    use fake::{Fake, Faker};
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn get_html_should_work() {
        // region: Arrange
        tracing_setup::span!("test");

        let ctx = text_context!(DbSource::Migration).await;
        let db = &ctx.db;

        let htmls = populate_htmls(db, 10).await;
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
        assert_eq!(result, html_to_get);
        // endregion: Assert
    }

    #[tokio::test]
    async fn upsert_htmls_should_work() {
        // region: Arrange
        tracing_setup::span!("test");

        let ctx = text_context!(DbSource::Migration).await;
        let db = &ctx.db;

        // Set up basic HTMLs
        let htmls = populate_htmls(db, 10).await;

        // Create HTMLs to upsert: from existing HTMLs
        const UPDATED_HTML_CONTENT: &str = "<html><body>Updated</body></html>";
        let mut updated_htmls = htmls
            .iter()
            .map(|h| Html {
                html: UPDATED_HTML_CONTENT.as_bytes().to_owned(),
                ..h.clone()
            })
            .collect::<Vec<_>>();

        // Add a new HTML to see that this HTML was properly inserted
        let mut new_html = Faker.fake::<Html>();
        const NEW_HTML_ID: &str = "2000000";
        new_html.smes_id = NEW_HTML_ID.to_string();
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
                    assert_eq!(html.html, UPDATED_HTML_CONTENT.as_bytes());
                }
            }
        }
        // endregion: Assert
    }

    async fn populate_htmls(db: &LibsqlDb, size: usize) -> Vec<Html> {
        let mut incremental_id: usize = 1000000;
        let htmls: Vec<Html> = (0..size)
            .map(|_| {
                let html = Faker.fake::<Html>();
                let id = incremental_id.to_string();
                incremental_id += 1;
                Html {
                    smes_id: id,
                    ..html
                }
            })
            .collect();

        let (tx, rx) = mpsc::unbounded_channel();
        for html in &htmls {
            tx.send(html.clone()).expect("Failed to send HTML");
        }
        drop(tx);

        db.insert_htmls(rx)
            .await
            .inspect_err(|e| {
                tracing::error!(?e, "Failed to insert HTMLs");
            })
            .expect("Failed to insert HTMLs");

        htmls
    }
}
