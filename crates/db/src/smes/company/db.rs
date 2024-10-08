use crate::db::Params;
use crate::error::ConversionError;
use crate::{DbError, LibsqlDb};
use hashbrown::HashSet;
use model::{company, db};
use serde::Deserialize;

impl LibsqlDb {
    #[tracing::instrument(skip(self))]
    pub async fn get_companies(&self) -> Result<Vec<db::Company>, DbError> {
        self.get_all_from::<crate::Company>("smes_company")
            .await?
            .into_iter()
            .map(|c| c.try_into())
            .collect()
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_company_ids(&self) -> Result<HashSet<company::Id>, DbError> {
        let mut rows = self
            .connection
            .query("SELECT smes_id from smes_company", ())
            .await?;
        let mut company_ids = HashSet::new();

        #[derive(Deserialize)]
        struct IdStruct {
            smes_id: String,
        }

        while let Some(row) = rows.next().await? {
            let id_struct: IdStruct = libsql::de::from_row(&row)?;
            company_ids.insert(id_struct.smes_id.try_into().map_err(ConversionError::new)?);
        }

        Ok(company_ids)
    }

    #[tracing::instrument(skip(self, companies))]
    /// Insert companies into the company table.
    ///
    /// The whole operation is processed in a single transaction.
    pub async fn insert_companies(&self, companies: Vec<db::Company>) -> Result<(), DbError> {
        let tx = self.connection.transaction().await?;
        let mut stmt = tx
            .prepare(
                "INSERT into smes_company (smes_id,
                     representative_name,
                     headquarters_address,
                     business_registration_number,
                     company_name,
                     industry_code,
                     industry_name)
VALUES (:smes_id,
        :representative_name,
        :headquarters_address,
        :business_registration_number,
        :company_name,
        :industry_code,
        :industry_name);",
            )
            .await?;

        for company in companies {
            stmt.execute(crate::Company::from(company).params()).await?;
            stmt.reset();
        }
        tx.commit().await?;

        Ok(())
    }

    #[tracing::instrument(skip(self, companies))]
    pub async fn upsert_companies(&self, companies: Vec<db::Company>) -> Result<(), DbError> {
        let tx = self.connection.transaction().await?;
        let mut stmt = tx
            .prepare(
                "INSERT into smes_company (smes_id,
                     representative_name,
                     headquarters_address,
                     business_registration_number,
                     company_name,
                     industry_code,
                     industry_name)
VALUES (:smes_id,
        :representative_name,
        :headquarters_address,
        :business_registration_number,
        :company_name,
        :industry_code,
        :industry_name)
ON CONFLICT (smes_id) DO UPDATE SET representative_name          = excluded.representative_name,
                               headquarters_address         = excluded.headquarters_address,
                               business_registration_number = excluded.business_registration_number,
                               company_name                 = excluded.company_name,
                               industry_code                = excluded.industry_code,
                               industry_name                = excluded.industry_name,
                               updated_date                  = CURRENT_DATE;",
            )
            .await?;

        for company in companies {
            stmt.execute(crate::Company::from(company).params()).await?;
            stmt.reset();
        }
        tx.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{text_context, DbSource, TestContext};
    use crate::LibsqlDb;
    use fake::Fake;
    use model::{company, db};
    use serde::Deserialize;
    use std::str::FromStr;

    #[test]
    fn id_struct_should_deserialize_as_expected() {
        #[derive(Deserialize)]
        struct IdStruct {
            id: company::Id,
        }

        let json = r#"{"id":"1234567"}"#;
        let id_struct: IdStruct = serde_json::from_str(json).unwrap();
        assert_eq!(id_struct.id, company::Id::from_str("1234567").unwrap());
    }

    #[tokio::test]
    async fn insert_and_get_companies_should_work() {
        tracing_setup::span!("test");

        let ctx = text_context!(DbSource::Migration).await;
        let db = &ctx.db;
        let companies = populate_companies(db, 10).await;

        let db_companies = db
            .get_companies()
            .await
            .inspect_err(|e| tracing::error!(?e, "Failed to get companies"))
            .unwrap();

        assert_eq!(db_companies.len(), companies.len());
    }

    #[tokio::test]
    async fn insert_and_get_company_ids_should_work() {
        tracing_setup::span!("test");

        let ctx = text_context!(DbSource::Migration).await;
        let db = &ctx.db;
        let companies = populate_companies(db, 10).await;

        let db_company_ids = db
            .get_company_ids()
            .await
            .inspect_err(|e| tracing::error!(?e, "Failed to get company ids"))
            .unwrap();

        let company_ids: hashbrown::HashSet<company::Id> =
            companies.into_iter().map(|c| c.smes_id).collect();
        tracing::trace!(?company_ids);
        assert_eq!(db_company_ids, company_ids);
    }

    #[tokio::test]
    async fn upsdert_companies_should_work() {
        // region: Arrange
        tracing_setup::span!("test");

        let ctx = text_context!(DbSource::Migration).await;
        let db = &ctx.db;

        // Set up basic companies
        let companies = populate_companies(db, 10).await;

        // Create companies to upsert: from existing companies.
        const UPDATED_REPRESENTATIVE_NAME: &str = "Updated";
        let mut updated_companies = companies
            .iter()
            .map(|c| db::Company {
                representative_name: UPDATED_REPRESENTATIVE_NAME.into(),
                ..c.clone()
            })
            .collect::<Vec<_>>();

        // Add a new company to see that this company was properly updated
        let mut new_company = ().fake::<db::Company>();
        const NEW_COMPANY_ID: &str = "2000000";
        new_company.smes_id = NEW_COMPANY_ID
            .try_into()
            .expect("failed to create dummy smes_id");
        let new_company_representative_name = new_company.representative_name.clone();
        updated_companies.push(new_company);

        // Remove a company to check that this company was not updated
        let removed_company = updated_companies.pop().unwrap();
        let removed_company_id = removed_company.smes_id.as_str();
        // endregion: Arrange

        // region: Action
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
            match company.smes_id.as_str() {
                NEW_COMPANY_ID => {
                    assert_eq!(company.representative_name, new_company_representative_name);
                }
                id if id == removed_company_id => {
                    // Not upserted company name should not change
                    assert_eq!(
                        company.representative_name,
                        removed_company.representative_name
                    );
                }
                _ => {
                    assert_eq!(
                        company.representative_name.as_str(),
                        UPDATED_REPRESENTATIVE_NAME
                    );
                }
            }
        }
        // endregion: Assert
    }

    async fn populate_companies(db: &LibsqlDb, size: usize) -> Vec<db::Company> {
        let mut incremental_id: usize = 1000000;
        let companies: Vec<db::Company> = (0..size)
            .map(|_| {
                let company = ().fake::<db::Company>();
                let id = incremental_id.to_string();
                incremental_id += 1;
                db::Company {
                    smes_id: id
                        .try_into()
                        .expect("failed to create proper dummy smes_id"),
                    ..company
                }
            })
            .collect();

        db.insert_companies(companies.clone())
            .await
            .inspect_err(|e| {
                tracing::error!(?e, "Failed to insert companies");
            })
            .unwrap();

        companies
    }
}
