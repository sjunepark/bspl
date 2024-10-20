use crate::DbError;
use hashbrown::HashSet;
use model::company;
use std::future::Future;

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
