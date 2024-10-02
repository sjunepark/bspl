use crate::smes::model::Params;
use crate::{Company, DbError, LibsqlDb};

impl LibsqlDb {
    #[tracing::instrument(skip(self))]
    pub async fn get_companies(&self) -> Result<Vec<Company>, DbError> {
        let mut rows = self.connection.query("SELECT * FROM Company", ()).await?;
        let mut companies = Vec::new();

        while let Some(row) = rows.next().await? {
            let company = libsql::de::from_row::<Company>(&row)?;
            companies.push(company);
        }

        Ok(companies)
    }

    #[tracing::instrument(skip(self, companies))]
    pub async fn insert_companies(&self, companies: &[Company]) -> Result<(), DbError> {
        let tx = self.connection.transaction().await?;
        let mut stmt = tx
            .prepare(
                "INSERT INTO Company (id,
                     representative_name,
                     headquarters_address,
                     business_registration_number,
                     company_name,
                     industry_code,
                     industry_name)
VALUES (:id,
        :representative_name,
        :headquarters_address,
        :business_registration_number,
        :company_name,
        :industry_code,
        :industry_name)",
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
    use fake::Fake;

    use crate::test_utils::{text_context, DbSource, TestContext};
    use crate::Company;

    #[tokio::test]
    async fn insert_and_get_companies_should_work() {
        tracing_setup::subscribe();

        let ctx = text_context!(DbSource::Migration).await;
        let db = &ctx.db;
        let mut incremental_id: usize = 1000000000;
        let companies: Vec<Company> = (0..10)
            .map(|_| {
                let company = ().fake::<Company>();
                let id = incremental_id.to_string();
                incremental_id += 1;
                Company { id, ..company }
            })
            .collect();

        db.insert_companies(&companies)
            .await
            .inspect_err(|e| tracing::error!(?e, "Failed to insert companies"))
            .unwrap();

        let db_companies = db
            .get_companies()
            .await
            .inspect_err(|e| tracing::error!(?e, "Failed to get companies"))
            .unwrap();

        assert_eq!(db_companies.len(), companies.len());
    }
}
