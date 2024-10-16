use crate::db::LibsqlParams;
use crate::smes::CompanyDb;
use crate::{DbError, LibsqlDb};
use hashbrown::HashSet;
use libsql::params::IntoParams;
use model::{company, table, ModelError};
use serde::{Deserialize, Serialize};

impl CompanyDb for LibsqlDb {
    #[tracing::instrument(skip(self))]
    async fn get_companies(&self) -> Result<Vec<table::Company>, DbError> {
        self.get_all_from::<crate::smes::LibsqlCompany>("smes_company")
            .await?
            .into_iter()
            .map(table::Company::try_from)
            .collect()
    }

    #[tracing::instrument(skip(self))]
    async fn get_company_ids(&self) -> Result<HashSet<company::Id>, DbError> {
        self.get_all_ids_from("smes_company").await
    }

    #[tracing::instrument(skip(self, companies))]
    /// Insert companies into the company table.
    ///
    /// The whole operation is processed in a single transaction.
    async fn insert_companies(&self, companies: Vec<table::Company>) -> Result<(), DbError> {
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
            stmt.execute(crate::smes::LibsqlCompany::from(company).params())
                .await?;
            stmt.reset();
        }
        tx.commit().await?;

        Ok(())
    }

    #[tracing::instrument(skip(self, companies))]
    async fn upsert_companies(&self, companies: Vec<table::Company>) -> Result<(), DbError> {
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
            stmt.execute(crate::smes::LibsqlCompany::from(company).params())
                .await?;
            stmt.reset();
        }
        tx.commit().await?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LibsqlCompany {
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

impl TryFrom<LibsqlCompany> for table::Company {
    type Error = DbError;

    fn try_from(value: LibsqlCompany) -> Result<Self, Self::Error> {
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

impl From<table::Company> for LibsqlCompany {
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

impl LibsqlParams for LibsqlCompany {
    fn params(&self) -> impl IntoParams {
        libsql::named_params! {
            ":smes_id": self.smes_id.as_str(),
            ":representative_name": self.representative_name.as_str(),
            ":headquarters_address": self.headquarters_address.as_str(),
            ":business_registration_number": self.business_registration_number.as_str(),
            ":company_name": self.company_name.as_str(),
            ":industry_code": self.industry_code.as_str(),
            ":industry_name": self.industry_name.as_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::smes::CompanyDb;
    use crate::test_utils::{LibsqlTestContext, TestContext};
    use fake::Fake;
    use model::{company, table};
    use serde::Deserialize;
    use std::str::FromStr;

    #[test]
    fn id_struct_should_deserialize_as_expected() {
        #[derive(Deserialize)]
        struct IdStruct {
            smes_id: company::Id,
        }

        let json = r#"{"smes_id":"1234567"}"#;
        let id_struct: IdStruct = serde_json::from_str(json).unwrap();
        assert_eq!(id_struct.smes_id, company::Id::from_str("1234567").unwrap());
    }

    #[tokio::test]
    async fn insert_and_get_companies_should_work() {
        tracing_setup::span!("test");

        let function_id = utils::function_id!();
        let ctx = LibsqlTestContext::new(&function_id).await;
        let db = ctx.db();

        let ids = (0..10_u64).map(|i| 1000000 + i).collect::<Vec<_>>();
        let companies = ctx.populate_companies(&ids).await;

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

        let function_id = utils::function_id!();
        let ctx = LibsqlTestContext::new(&function_id).await;
        let db = ctx.db();
        let ids = (0..10_u64).map(|i| 1000000 + i).collect::<Vec<_>>();
        let companies = ctx.populate_companies(&ids).await;

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
    async fn upsert_companies_should_work() {
        // region: Arrange
        tracing_setup::span!("test");

        let function_id = utils::function_id!();
        let ctx = LibsqlTestContext::new(&function_id).await;
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
