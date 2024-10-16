use crate::{DbError, PostgresqlDb};
use hashbrown::HashSet;
use model::company::Id;
use model::table::Company;
use model::{company, table};
use std::future::Future;

pub trait CompanyDb {
    fn get_companies(&self) -> impl Future<Output = Result<Vec<table::Company>, DbError>>;
    fn get_company_ids(&self) -> impl Future<Output = Result<HashSet<company::Id>, DbError>>;
    fn insert_companies(
        &self,
        companies: Vec<table::Company>,
    ) -> impl Future<Output = Result<(), DbError>>;
    fn upsert_companies(
        &self,
        companies: Vec<table::Company>,
    ) -> impl Future<Output = Result<(), DbError>>;
}

impl CompanyDb for PostgresqlDb {
    async fn get_companies(&self) -> Result<Vec<Company>, DbError> {
        todo!()
    }

    async fn get_company_ids(&self) -> Result<HashSet<Id>, DbError> {
        todo!()
    }

    async fn insert_companies(&self, companies: Vec<Company>) -> Result<(), DbError> {
        todo!()
    }

    async fn upsert_companies(&self, companies: Vec<Company>) -> Result<(), DbError> {
        todo!()
    }
}
