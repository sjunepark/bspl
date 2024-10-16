use crate::smes::CompanyDb;
use crate::{DbError, PostgresDb};
use hashbrown::HashSet;
use model::{company, table, ModelError};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

impl CompanyDb for PostgresDb {
    async fn get_companies(&self) -> Result<Vec<table::Company>, DbError> {
        sqlx::query_as!(PostgresCompany, "SELECT * from smes_company")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(table::Company::try_from)
            .collect()
    }

    async fn get_company_ids(&self) -> Result<HashSet<company::Id>, DbError> {
        sqlx::query!("SELECT smes_id from smes_company")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|company| {
                company::Id::try_from(company.smes_id)
                    .map_err(|e| DbError::from(ModelError::from(e)))
            })
            .collect()
    }

    async fn insert_companies(&self, companies: Vec<table::Company>) -> Result<(), DbError> {
        let mut tx = self.pool.begin().await?;
        let total_company_count = companies.len() as u64;
        let mut insert_count = 0_u64;

        for company in companies {
            let result = sqlx::query!(
                "INSERT INTO smes_company (smes_id, representative_name, headquarters_address, business_registration_number,
                          company_name, industry_code, industry_name)
VALUES ($1, $2, $3, $4, $5, $6, $7)",
                company.smes_id.to_string(),
                company.representative_name.to_string(),
                company.headquarters_address.to_string(),
                company.business_registration_number.to_string(),
                company.company_name.to_string(),
                company.industry_code.to_string(),
                company.industry_name.to_string(),
            ).execute(&mut *tx).await?.rows_affected();
            insert_count += result;
        }

        if insert_count == total_company_count {
            tracing::trace!("Inserted {} companies", insert_count);
            tx.commit().await?;
        } else {
            tracing::error!(
                "Inserted {}/{} companies. Rolling back transaction",
                insert_count,
                total_company_count
            );
            tx.rollback().await?;
        }
        Ok(())
    }

    async fn upsert_companies(&self, companies: Vec<table::Company>) -> Result<(), DbError> {
        let mut tx = self.pool.begin().await?;
        let total_company_count = companies.len() as u64;
        let mut upsert_count = 0_u64;

        for company in companies {
            let result = sqlx::query!(
                r#"INSERT INTO smes_company (smes_id, representative_name, headquarters_address, business_registration_number,
                          company_name, industry_code, industry_name)
VALUES ($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT (smes_id) DO UPDATE SET representative_name          = $2,
                                    headquarters_address         = $3,
                                    business_registration_number = $4,
                                    company_name                 = $5,
                                    industry_code                = $6,
                                    industry_name                = $7,
                                    updated_date                 = DEFAULT"#,
                company.smes_id.to_string(),
                company.representative_name.to_string(),
                company.headquarters_address.to_string(),
                company.business_registration_number.to_string(),
                company.company_name.to_string(),
                company.industry_code.to_string(),
                company.industry_name.to_string(),
            ).execute(&mut *tx).await?.rows_affected();
            upsert_count += result;
        }

        if upsert_count == total_company_count {
            tracing::trace!("Upserted {} companies", upsert_count);
            tx.commit().await?;
        } else {
            tracing::error!(
                "Upserted {}/{} companies. Rolling back transaction",
                upsert_count,
                total_company_count
            );
            tx.rollback().await?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct PostgresCompany {
    pub smes_id: String,
    pub representative_name: String,
    pub headquarters_address: String,
    pub business_registration_number: String,
    pub company_name: String,
    pub industry_code: String,
    pub industry_name: String,
    pub created_date: Option<time::Date>,
    pub updated_date: Option<time::Date>,
}

impl TryFrom<PostgresCompany> for table::Company {
    type Error = DbError;

    fn try_from(value: PostgresCompany) -> Result<Self, Self::Error> {
        Ok(table::Company {
            smes_id: company::Id::try_from(value.smes_id).map_err(ModelError::from)?,
            representative_name: Into::<company::RepresentativeName>::into(
                value.representative_name,
            ),
            headquarters_address: Into::<company::HeadquartersAddress>::into(
                value.headquarters_address,
            ),
            business_registration_number: company::BusinessRegistrationNumber::try_from(
                value.business_registration_number,
            )
            .map_err(ModelError::from)?,
            company_name: Into::<company::CompanyName>::into(value.company_name),
            industry_code: company::IndustryCode::try_from(value.industry_code)
                .map_err(ModelError::from)?,
            industry_name: Into::<company::IndustryName>::into(value.industry_name),
            created_date: value.created_date,
            updated_date: value.updated_date,
        })
    }
}

impl From<table::Company> for PostgresCompany {
    fn from(value: table::Company) -> Self {
        Self {
            smes_id: value.smes_id.to_string(),
            representative_name: value.representative_name.to_string(),
            headquarters_address: value.headquarters_address.to_string(),
            business_registration_number: value.business_registration_number.to_string(),
            company_name: value.company_name.to_string(),
            industry_code: value.industry_code.to_string(),
            industry_name: value.industry_name.to_string(),
            created_date: value.created_date,
            updated_date: value.updated_date,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::smes::CompanyDb;
    use crate::test_utils::{PostgresTestContext, TestContext};
    use fake::Fake;
    use hashbrown::HashSet;
    use model::table;

    #[tokio::test]
    async fn insert_and_get_companies_should_work() {
        let function_id = utils::function_id!();
        let ctx = PostgresTestContext::new(&function_id).await;

        let ids = (0..10_u64).map(|i| 1000000 + i).collect::<Vec<_>>();
        let inserted_companies_without_date = ctx.populate_companies(&ids).await;

        let selected_companies: Vec<_> = ctx
            .db()
            .get_companies()
            .await
            .expect("Failed to get companies")
            .into_iter()
            .map(|company| table::Company {
                created_date: None,
                updated_date: None,
                ..company
            })
            .collect();

        assert_eq!(inserted_companies_without_date, selected_companies);
    }

    #[tokio::test]
    async fn get_company_ids_should_work() {
        let function_id = utils::function_id!();
        let ctx = PostgresTestContext::new(&function_id).await;

        let ids = (0..10_u64).map(|i| 1000000 + i).collect::<Vec<_>>();
        let inserted_ids: HashSet<_> = ctx
            .populate_companies(&ids)
            .await
            .into_iter()
            .map(|company| company.smes_id)
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
        let ctx = PostgresTestContext::new(&function_id).await;
        let db = ctx.db();

        // Set up basic companies
        let ids = (0..10_u64).map(|i| 1000000 + i).collect::<Vec<_>>();
        let companies = ctx.populate_companies(&ids).await;

        // Create companies to upsert: from existing companies.
        const UPDATED_REPRESENTATIVE_NAME: &str = "Updated";
        let mut updated_companies = companies
            .iter()
            .map(|c| table::Company {
                representative_name: UPDATED_REPRESENTATIVE_NAME.into(),
                ..c.clone()
            })
            .collect::<Vec<_>>();

        // Add a new company to see that this company was properly updated
        let mut new_company = ().fake::<table::Company>();
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
}
