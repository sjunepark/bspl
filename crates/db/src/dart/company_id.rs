use crate::entities::dart::company_id;
use crate::entities::dart::prelude::*;
use crate::{DbError, PostgresDb, POSTGRES_MAX_PARAMETERS};
use sea_orm::sea_query::OnConflict;
use sea_orm::EntityTrait;
use std::future::Future;

pub trait CompanyIdDb {
    fn get_company_ids(&mut self) -> impl Future<Output = Result<Vec<company_id::Model>, DbError>>;
    fn insert_company_ids(
        &mut self,
        company_ids: Vec<company_id::ActiveModel>,
    ) -> impl Future<Output = Result<(), DbError>>;
    fn upsert_company_ids(
        &mut self,
        company_ids: Vec<company_id::ActiveModel>,
    ) -> impl Future<Output = Result<(), DbError>>;
}

impl CompanyIdDb for PostgresDb {
    #[tracing::instrument(skip(self))]
    async fn get_company_ids(&mut self) -> Result<Vec<company_id::Model>, DbError> {
        Ok(crate::entities::dart::prelude::CompanyId::find()
            .all(&self.conn)
            .await?)
    }

    #[tracing::instrument(skip(self, company_ids))]
    async fn insert_company_ids(
        &mut self,
        company_ids: Vec<company_id::ActiveModel>,
    ) -> Result<(), DbError> {
        const BUFFER_DIVISOR: usize = 100;

        for chunk in company_ids.chunks(POSTGRES_MAX_PARAMETERS / BUFFER_DIVISOR) {
            tracing::trace!(chunk_size = chunk.len(), "Inserting chunk of company_ids");
            self.insert_company_ids_inner(chunk.to_vec()).await?;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self, company_ids))]
    async fn upsert_company_ids(
        &mut self,
        company_ids: Vec<company_id::ActiveModel>,
    ) -> Result<(), DbError> {
        const BUFFER_DIVISOR: usize = 100;

        for chunk in company_ids.chunks(POSTGRES_MAX_PARAMETERS / BUFFER_DIVISOR) {
            tracing::trace!(chunk_size = chunk.len(), "Upserting chunk of company_ids");
            self.upsert_company_ids_inner(chunk.to_vec()).await?;
        }
        Ok(())
    }
}

impl PostgresDb {
    async fn insert_company_ids_inner(
        &mut self,
        company_ids: Vec<company_id::ActiveModel>,
    ) -> Result<(), DbError> {
        let total_company_id_count = company_ids.len();
        let res = CompanyId::insert_many(company_ids).exec(&self.conn).await?;
        tracing::info!(?res.last_insert_id, "Inserted {} company_ids", total_company_id_count);
        Ok(())
    }

    async fn upsert_company_ids_inner(
        &mut self,
        company_ids: Vec<company_id::ActiveModel>,
    ) -> Result<(), DbError> {
        let total_company_id_count = company_ids.len();

        let on_conflict = OnConflict::column(company_id::Column::DartId)
            .update_columns(vec![
                company_id::Column::CompanyName,
                company_id::Column::StockCode,
                company_id::Column::IdModifyDate,
            ])
            .to_owned();

        let res = CompanyId::insert_many(company_ids)
            .on_conflict(on_conflict)
            .exec(&self.conn)
            .await?;
        tracing::info!(?res.last_insert_id, "Upserted {} company_ids", total_company_id_count);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::dart::CompanyIdDb;
    use crate::entities::dart::company_id;
    use crate::model::dart::CompanyId;
    use crate::test_utils::{PostgresTestContext, TestContext};
    use fake::{Fake, Faker};
    use sea_orm::IntoActiveModel;

    #[tokio::test]
    async fn insert_and_get_company_ids_should_work() {
        let function_id = utils::function_id!();
        let mut ctx = PostgresTestContext::new(&function_id).await;

        let ids = (0..10_u64).map(|i| 10000000 + i).collect::<Vec<_>>();
        let mut inserted_company_ids = ctx.populate_company_ids(&ids).await;

        let mut selected_company_ids: Vec<_> = ctx
            .db()
            .get_company_ids()
            .await
            .expect("Failed to get company_ids")
            .into_iter()
            .collect();

        inserted_company_ids.sort_by_key(|c| c.dart_id.clone());
        selected_company_ids.sort_by_key(|c| c.dart_id.clone());

        assert_eq!(inserted_company_ids, selected_company_ids,);
    }

    #[tokio::test]
    async fn upsert_company_ids_should_work() {
        tracing_setup::span!("test");

        let function_id = utils::function_id!();
        let mut ctx = PostgresTestContext::new(&function_id).await;

        let ids = (0..10_u64).map(|i| 10000000 + i).collect::<Vec<_>>();
        let inserted_company_ids = ctx.populate_company_ids(&ids).await;

        // Create companies to upsert: from existing companies.
        const UPDATED_COMPANY_NAME: &str = "Updated";
        let mut updated_company_ids = inserted_company_ids
            .iter()
            .map(|company_id| company_id::Model {
                company_name: UPDATED_COMPANY_NAME.to_string(),
                ..company_id.clone()
            })
            .collect::<Vec<_>>();

        // Add a new company to see that this company was properly updated
        let mut new_company_id: CompanyId = Faker.fake::<CompanyId>();
        const NEW_DART_ID: &str = "20000000";
        new_company_id.dart_id = NEW_DART_ID.try_into().expect("Failed to convert");
        let new_company_name = new_company_id.company_name.clone();
        updated_company_ids.push(company_id::Model::from(new_company_id));

        // Remove a company to check that this company was not updated
        let removed_company_id = updated_company_ids.pop().unwrap();
        let removed_dart_id = removed_company_id.dart_id;
        // endregion: Arrange

        // region: Action
        let db = ctx.db();
        db.upsert_company_ids(
            updated_company_ids
                .into_iter()
                .map(|company_id| company_id.into_active_model())
                .collect(),
        )
        .await
        .inspect_err(|e| tracing::error!(?e, "Failed to upsert companies"))
        .unwrap();
        // endregion: Action

        // region: Assert
        let db_company_ids = db
            .get_company_ids()
            .await
            .inspect_err(|e| tracing::error!(?e, "Failed to get company_ids"))
            .unwrap();

        for company_id in &db_company_ids {
            match company_id.dart_id.as_ref().as_str() {
                NEW_DART_ID => {
                    assert_eq!(&company_id.company_name, new_company_name.as_str());
                }
                id if id == removed_dart_id.as_ref() => {
                    // Not upserted company name should not change
                    assert_eq!(company_id.company_name, removed_company_id.company_name);
                }
                _ => {
                    assert_eq!(company_id.company_name.as_str(), UPDATED_COMPANY_NAME,);
                }
            }
        }
        // endregion: Assert
    }
}
