use diesel::{Insertable, Queryable, Selectable};
use fake::faker::address::ja_jp::CityName;
use fake::faker::name::ja_jp::Name;
use fake::faker::number::raw::NumberWithFormat;
use fake::locales::EN;
use fake::{Dummy, Fake};
use rand::Rng;
use types::company;

impl<T> Dummy<T> for crate::model::smes::Filing {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &T, rng: &mut R) -> Self {
        let fake_time =
            fake::faker::time::en::DateTime().fake_with_rng::<time::PrimitiveDateTime, R>(rng);

        let new_filing = crate::model::smes::NewFiling::dummy_with_rng(_config, rng);

        crate::model::smes::Filing {
            filing_id: new_filing.filing_id,
            representative_name: new_filing.representative_name,
            headquarters_address: new_filing.headquarters_address,
            business_registration_number: new_filing.business_registration_number,
            filing_name: new_filing.filing_name,
            industry_code: new_filing.industry_code,
            industry_name: new_filing.industry_name,
            created_at: fake_time,
            updated_at: fake_time,
        }
    }
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = crate::schema::dart::dart::filing)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewFiling {
    pub filing_id: filing::Id,
    pub representative_name: filing::RepresentativeName,
    pub headquarters_address: filing::HeadquartersAddress,
    pub business_registration_number: filing::BusinessRegistrationNumber,
    pub filing_name: filing::Name,
    pub industry_code: filing::IndustryCode,
    pub industry_name: filing::IndustryName,
}

impl<T> Dummy<T> for crate::model::smes::NewFiling {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &T, rng: &mut R) -> Self {
        crate::model::smes::NewFiling {
            filing_id: NumberWithFormat(EN, "^######")
                .fake::<String>()
                .as_str()
                .try_into()
                .expect("dummy creation logic needs to be fixed within the source code"),
            representative_name: Name().fake_with_rng::<String, R>(rng).into(),
            headquarters_address: format!(
                "{}, South Korea",
                CityName().fake_with_rng::<String, R>(rng)
            )
            .into(),
            business_registration_number: NumberWithFormat(EN, "^#########")
                .fake::<String>()
                .as_str()
                .try_into()
                .expect("dummy creation logic needs to be fixed within the source code"),
            filing_name: FilingName().fake_with_rng::<String, R>(rng).into(),
            industry_code: NumberWithFormat(EN, "^####")
                .fake::<String>()
                .as_str()
                .try_into()
                .expect("dummy creation logic needs to be fixed within the source code"),
            industry_name: Industry().fake_with_rng::<String, R>(rng).into(),
        }
    }
}

impl From<Filing> for NewFiling {
    fn from(filing: Filing) -> Self {
        NewFiling {
            filing_id: filing.filing_id,
            representative_name: filing.representative_name,
            headquarters_address: filing.headquarters_address,
            business_registration_number: filing.business_registration_number,
            filing_name: filing.filing_name,
            industry_code: filing.industry_code,
            industry_name: filing.industry_name,
        }
    }
}

impl PartialEq for NewFiling {
    fn eq(&self, other: &Self) -> bool {
        self.filing_id == other.filing_id
            && self.representative_name == other.representative_name
            && self.headquarters_address == other.headquarters_address
            && self.business_registration_number == other.business_registration_number
            && self.filing_name == other.filing_name
            && self.industry_code == other.industry_code
            && self.industry_name == other.industry_name
    }
}
// endregion: Table filing
