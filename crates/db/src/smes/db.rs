use crate::smes::model::Params;
use crate::{Company, DbError, LibsqlDb};
use hashbrown::HashSet;
use serde::Deserialize;

impl LibsqlDb {
    #[tracing::instrument(skip(self))]
    pub async fn get_companies(&self) -> Result<Vec<Company>, DbError> {
        let mut rows = self
            .connection
            .query("SELECT * from smes_company", ())
            .await?;
        let mut companies = Vec::new();

        while let Some(row) = rows.next().await? {
            let a = row.get_value(0).expect("Failed to get value");
            tracing::info!(?a);
            let company = libsql::de::from_row::<Company>(&row)?;
            companies.push(company);
        }

        Ok(companies)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_company_ids(&self) -> Result<HashSet<String>, DbError> {
        let mut rows = self
            .connection
            .query("SELECT id from smes_company", ())
            .await?;
        let mut company_ids = HashSet::new();

        #[derive(Deserialize)]
        struct IdStruct {
            id: String,
        }

        while let Some(row) = rows.next().await? {
            let id_struct: IdStruct = libsql::de::from_row(&row)?;
            company_ids.insert(id_struct.id);
        }

        Ok(company_ids)
    }

    #[tracing::instrument(skip(self, companies))]
    pub async fn insert_companies(&self, companies: &[Company]) -> Result<(), DbError> {
        let tx = self.connection.transaction().await?;
        let mut stmt = tx
            .prepare(
                "INSERT into smes_company (id,
                     representative_name,
                     headquarters_address,
                     business_registration_number,
                     company_name,
                     industry_code,
                     industry_name,
                     create_date,
                     update_date)
VALUES (:id,
        :representative_name,
        :headquarters_address,
        :business_registration_number,
        :company_name,
        :industry_code,
        :industry_name,
        :create_date,
        :update_date);",
            )
            .await?;

        for company in companies {
            stmt.execute(company.params()).await?;
            stmt.reset();
        }
        tx.commit().await?;

        Ok(())
    }

    #[tracing::instrument(skip(self, companies))]
    pub async fn upsert_companies(&self, companies: &[Company]) -> Result<(), DbError> {
        let tx = self.connection.transaction().await?;
        let mut stmt = tx
            .prepare(
                "INSERT into smes_company (id,
                     representative_name,
                     headquarters_address,
                     business_registration_number,
                     company_name,
                     industry_code,
                     industry_name,
                     create_date,
                     update_date)
VALUES (:id,
        :representative_name,
        :headquarters_address,
        :business_registration_number,
        :company_name,
        :industry_code,
        :industry_name,
        :create_date,
        :update_date)
ON CONFLICT (id) DO UPDATE SET representative_name          = EXCLUDED.representative_name,
                               headquarters_address         = EXCLUDED.headquarters_address,
                               business_registration_number = EXCLUDED.business_registration_number,
                               company_name                 = EXCLUDED.company_name,
                               industry_code                = EXCLUDED.industry_code,
                               industry_name                = EXCLUDED.industry_name,
                               update_date                  = EXCLUDED.update_date;",
            )
            .await?;

        for company in companies {
            stmt.execute(company.params()).await?;
            stmt.reset();
        }
        tx.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{text_context, DbSource, TestContext};
    use crate::{Company, LibsqlDb};
    use fake::Fake;
    use model::company;
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

        let company_ids: hashbrown::HashSet<String> = companies.into_iter().map(|c| c.id).collect();
        tracing::trace!(?company_ids);
        assert_eq!(db_company_ids, company_ids);
    }

    #[tokio::test]
    async fn upsert_companies_should_work() {
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
            .map(|c| Company {
                representative_name: UPDATED_REPRESENTATIVE_NAME.to_string(),
                ..c.clone()
            })
            .collect::<Vec<_>>();

        // Add a new company to see that this company was properly updated
        let mut new_company = ().fake::<Company>();
        const NEW_COMPANY_ID: &str = "2000000";
        new_company.id = NEW_COMPANY_ID.to_string();
        let new_company_representative_name = new_company.representative_name.clone();
        updated_companies.push(new_company);

        // Remove a company to check that this company was not updated
        let removed_company = updated_companies.pop().unwrap();
        let removed_company_id = removed_company.id.as_str();
        // endregion: Arrange

        // region: Action
        db.upsert_companies(&updated_companies)
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
            match company.id.as_str() {
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

    async fn populate_companies(db: &LibsqlDb, size: usize) -> Vec<Company> {
        let mut incremental_id: usize = 1000000;
        let companies: Vec<Company> = (0..size)
            .map(|_| {
                let company = ().fake::<Company>();
                let id = incremental_id.to_string();
                incremental_id += 1;
                Company { id, ..company }
            })
            .collect();

        db.insert_companies(&companies)
            .await
            .inspect_err(|e| {
                tracing::error!(?e, "Failed to insert companies");
            })
            .unwrap();

        companies
    }
}
