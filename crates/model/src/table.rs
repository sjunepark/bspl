use crate::company;
use chrono::NaiveDate;
use fake::faker::address::ja_jp::CityName;
use fake::faker::company::ja_jp::{CompanyName, Industry};
use fake::faker::name::ja_jp::Name;
use fake::faker::number::raw::NumberWithFormat;
use fake::locales::EN;
use fake::{Dummy, Fake, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Company {
    pub smes_id: company::Id,
    pub representative_name: company::RepresentativeName,
    pub headquarters_address: company::HeadquartersAddress,
    pub business_registration_number: company::BusinessRegistrationNumber,
    pub company_name: company::CompanyName,
    pub industry_code: company::IndustryCode,
    pub industry_name: company::IndustryName,
    pub created_date: Option<NaiveDate>,
    pub updated_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Html {
    pub smes_id: company::Id,
    pub html: company::HtmlContent,
    pub created_date: Option<NaiveDate>,
    pub updated_date: Option<NaiveDate>,
}

impl<T> Dummy<T> for Html {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &T, rng: &mut R) -> Self {
        Html {
            smes_id: NumberWithFormat(EN, "^######")
                .fake::<String>()
                .try_into()
                .expect("failed to create dummy smes_id"),
            html: format!(
                "<html><head><title>{}</title></head><body>{}</body></html>",
                CompanyName().fake_with_rng::<String, R>(rng),
                Name().fake_with_rng::<String, R>(rng)
            )
            .as_bytes()
            .to_vec()
            .into(),
            created_date: None,
            updated_date: None,
        }
    }
}

impl<T> Dummy<T> for Company {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &T, rng: &mut R) -> Self {
        Company {
            smes_id: NumberWithFormat(EN, "^######")
                .fake::<String>()
                .try_into()
                .expect("failed to create dummy smes_id"),
            representative_name: Name().fake_with_rng::<String, R>(rng).into(),
            headquarters_address: format!(
                "{}, South Korea",
                CityName().fake_with_rng::<String, R>(rng)
            )
            .into(),
            business_registration_number: NumberWithFormat(EN, "^#########")
                .fake::<String>()
                .try_into()
                .expect("failed to create dummy business_registration_number"),
            company_name: CompanyName().fake_with_rng::<String, R>(rng).into(),
            industry_code: NumberWithFormat(EN, "^####")
                .fake::<String>()
                .try_into()
                .expect("failed to create dummy industry_code"),
            industry_name: Industry().fake_with_rng::<String, R>(rng).into(),
            created_date: None,
            updated_date: None,
        }
    }
}
