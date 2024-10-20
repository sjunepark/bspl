use crate::company;
use fake::faker::address::ja_jp::CityName;
use fake::faker::company::ja_jp::{CompanyName, Industry};
use fake::faker::name::ja_jp::Name;
use fake::faker::number::raw::NumberWithFormat;
use fake::locales::EN;
use fake::{Dummy, Fake, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Company {
    pub company_id: company::Id,
    pub representative_name: company::RepresentativeName,
    pub headquarters_address: company::HeadquartersAddress,
    pub business_registration_number: company::BusinessRegistrationNumber,
    pub company_name: company::CompanyName,
    pub industry_code: company::IndustryCode,
    pub industry_name: company::IndustryName,
    pub created_at: Option<time::PrimitiveDateTime>,
    pub updated_at: Option<time::PrimitiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Html {
    pub company_id: company::Id,
    pub html: company::HtmlContent,
    pub created_at: Option<time::PrimitiveDateTime>,
    pub updated_at: Option<time::PrimitiveDateTime>,
}

impl<T> Dummy<T> for Html {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &T, rng: &mut R) -> Self {
        Html {
            company_id: NumberWithFormat(EN, "^######")
                .fake::<String>().as_str()
                .try_into()
                .expect("failed to create dummy company_id"),
            html: format!(
                "<html><head><title>{}</title></head><body><h2>유동자산</h2><p>{}</p></body></html>",
                CompanyName().fake_with_rng::<String, R>(rng),
                Name().fake_with_rng::<String, R>(rng)
            )
            .try_into().expect("failed to create dummy html"),
            created_at: None,
            updated_at: None,
        }
    }
}

impl<T> Dummy<T> for Company {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &T, rng: &mut R) -> Self {
        Company {
            company_id: NumberWithFormat(EN, "^######")
                .fake::<String>()
                .as_str()
                .try_into()
                .expect("failed to create dummy company_id"),
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
            created_at: None,
            updated_at: None,
        }
    }
}
