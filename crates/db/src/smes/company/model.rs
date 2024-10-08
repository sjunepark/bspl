use crate::db::Params;
use crate::DbError;
use chrono::NaiveDate;
use libsql::params::IntoParams;
use model::{company, table, ModelError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Company {
    pub smes_id: String,
    pub representative_name: String,
    pub headquarters_address: String,
    pub business_registration_number: String,
    pub company_name: String,
    pub industry_code: String,
    pub industry_name: String,
    pub created_date: Option<NaiveDate>,
    pub updated_date: Option<NaiveDate>,
}

impl TryFrom<Company> for table::Company {
    type Error = DbError;

    fn try_from(value: Company) -> Result<Self, Self::Error> {
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

impl From<table::Company> for Company {
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

impl Params for Company {
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
