use crate::DbError;
use hashbrown::HashSet;
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
