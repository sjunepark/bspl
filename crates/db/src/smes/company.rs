use crate::schema::smes::company::dsl;
use crate::{DbError, PostgresDb};
use diesel::prelude::*;
use diesel::upsert::excluded;
use hashbrown::HashSet;
use std::future::Future;
use types::company;

impl CompanyDb for PostgresDb {
    async fn get_companies(&mut self) -> Result<Vec<crate::model::smes::Company>, DbError> {
        Ok(dsl::company.load(&mut self.conn)?)
    }

    async fn get_company_ids(&mut self) -> Result<HashSet<company::Id>, DbError> {
        dsl::company
            .select(dsl::company_id)
            .load::<String>(&mut self.conn)?
            .into_iter()
            .map(|id| company::Id::try_from(id.as_str()).map_err(DbError::from))
            .collect::<Result<HashSet<_>, _>>()
    }

    async fn insert_companies(
        &mut self,
        companies: Vec<crate::model::smes::NewCompany>,
    ) -> Result<(), DbError> {
        let total_company_count = companies.len() as u64;

        self.conn.transaction::<(), _, _>(|conn| {
            let insert_count = diesel::insert_into(dsl::company)
                .values(&companies)
                .execute(conn)?;

            if insert_count == total_company_count as usize {
                tracing::trace!("Inserted {} companies", insert_count);
                Ok(())
            } else {
                tracing::error!(
                    "Inserted {}/{} companies. Rolling back transaction",
                    insert_count,
                    total_company_count
                );
                Err(diesel::result::Error::RollbackTransaction)
            }
        })?;
        Ok(())
    }

    async fn upsert_companies(
        &mut self,
        companies: Vec<crate::model::smes::NewCompany>,
    ) -> Result<(), DbError> {
        self.conn.transaction(|conn| {
            let insert_count = diesel::insert_into(dsl::company)
                .values(&companies)
                .on_conflict(dsl::company_id)
                .do_update()
                .set((
                    dsl::representative_name.eq(excluded(dsl::representative_name)),
                    dsl::headquarters_address.eq(excluded(dsl::headquarters_address)),
                    dsl::business_registration_number
                        .eq(excluded(dsl::business_registration_number)),
                    dsl::company_name.eq(excluded(dsl::company_name)),
                    dsl::industry_code.eq(excluded(dsl::industry_code)),
                    dsl::industry_name.eq(excluded(dsl::industry_name)),
                ))
                .execute(conn)?;

            if insert_count == companies.len() {
                tracing::trace!("Upserted {} companies", insert_count);
                Ok(())
            } else {
                tracing::error!(
                    "Upserted {}/{} companies. Rolling back transaction",
                    insert_count,
                    companies.len()
                );
                Err(diesel::result::Error::RollbackTransaction)
            }
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::model::smes::NewCompany;
    use crate::smes::company::CompanyDb;
    use crate::test_utils::{PostgresTestContext, TestContext};
    use fake::Fake;
    use hashbrown::HashSet;

    #[tokio::test]
    async fn insert_and_get_companies_should_work() {
        let function_id = utils::function_id!();
        let mut ctx = PostgresTestContext::new(&function_id).await;

        let ids = (0..10_u64).map(|i| 1000000 + i).collect::<Vec<_>>();
        let mut inserted_companies = ctx.populate_companies(&ids).await;

        let mut selected_companies: Vec<_> = ctx
            .db()
            .get_companies()
            .await
            .expect("Failed to get companies")
            .into_iter()
            .map(NewCompany::from)
            .collect();

        inserted_companies.sort_by_key(|c| c.company_id.clone());
        selected_companies.sort_by_key(|c| c.company_id.clone());

        assert_eq!(inserted_companies, selected_companies,);
    }

    #[tokio::test]
    async fn get_company_ids_should_work() {
        let function_id = utils::function_id!();
        let mut ctx = PostgresTestContext::new(&function_id).await;

        let ids = (0..10_u64).map(|i| 1000000 + i).collect::<Vec<_>>();
        let inserted_ids: HashSet<_> = ctx
            .populate_companies(&ids)
            .await
            .into_iter()
            .map(|company| company.company_id)
            .collect();

        let selected_ids = ctx
            .db()
            .get_company_ids()
            .await
            .expect("Failed to get company ids");

        assert_eq!(inserted_ids, selected_ids);
    }

    #[tokio::test]
    async fn upsert_companies_should_work() {
        // region: Arrange
        tracing_setup::span!("test");

        let function_id = utils::function_id!();
        let mut ctx = PostgresTestContext::new(&function_id).await;

        // Set up basic companies
        let ids = (0..10_u64).map(|i| 1000000 + i).collect::<Vec<_>>();
        let companies = ctx.populate_companies(&ids).await;

        // Create companies to upsert: from existing companies.
        const UPDATED_REPRESENTATIVE_NAME: &str = "Updated";
        let mut updated_companies = companies
            .iter()
            .map(|c| NewCompany {
                representative_name: UPDATED_REPRESENTATIVE_NAME.to_string().into(),
                ..c.clone()
            })
            .collect::<Vec<_>>();

        // Add a new company to see that this company was properly updated
        let mut new_company = ().fake::<NewCompany>();
        const NEW_COMPANY_ID: &str = "2000000";
        new_company.company_id = NEW_COMPANY_ID
            .try_into()
            .expect("failed to create dummy company_id");
        let new_company_representative_name = new_company.representative_name.clone();
        updated_companies.push(new_company);

        // Remove a company to check that this company was not updated
        let removed_company = updated_companies.pop().unwrap();
        let removed_company_id = removed_company.company_id;
        // endregion: Arrange

        // region: Action
        let db = ctx.db();
        db.upsert_companies(updated_companies)
            .await
            .inspect_err(|e| tracing::error!(?e, "Failed to upsert companies"))
            .unwrap();
        // endregion: Action

        // region: Assert
        let db_companies = db
            .get_companies()
            .await
            .inspect_err(|e| tracing::error!(?e, "Failed to get companies"))
            .unwrap();

        for company in &db_companies {
            match company.company_id.as_ref().as_str() {
                NEW_COMPANY_ID => {
                    assert_eq!(company.representative_name, new_company_representative_name);
                }
                id if id == removed_company_id.as_ref() => {
                    // Not upserted company name should not change
                    assert_eq!(
                        company.representative_name,
                        removed_company.representative_name
                    );
                }
                _ => {
                    assert_eq!(
                        company.representative_name.as_ref(),
                        UPDATED_REPRESENTATIVE_NAME
                    );
                }
            }
        }
        // endregion: Assert
    }
}

pub trait CompanyDb {
    fn get_companies(
        &mut self,
    ) -> impl Future<Output = Result<Vec<crate::model::smes::Company>, DbError>>;
    fn get_company_ids(&mut self) -> impl Future<Output = Result<HashSet<company::Id>, DbError>>;
    fn insert_companies(
        &mut self,
        companies: Vec<crate::model::smes::NewCompany>,
    ) -> impl Future<Output = Result<(), DbError>>;
    fn upsert_companies(
        &mut self,
        companies: Vec<crate::model::smes::NewCompany>,
    ) -> impl Future<Output = Result<(), DbError>>;
}
