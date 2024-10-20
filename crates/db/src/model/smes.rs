use diesel::{Insertable, Queryable, Selectable};
use fake::faker::address::ja_jp::CityName;
use fake::faker::company::ja_jp::{CompanyName, Industry};
use fake::faker::name::ja_jp::Name;
use fake::faker::number::raw::NumberWithFormat;
use fake::locales::EN;
use fake::{Dummy, Fake};
use rand::Rng;

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::smes::company)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Company {
    pub company_id: model::company::Id,
    pub representative_name: String,
    pub headquarters_address: String,
    pub business_registration_number: String,
    pub company_name: String,
    pub industry_code: String,
    pub industry_name: String,
    pub created_at: time::PrimitiveDateTime,
    pub updated_at: time::PrimitiveDateTime,
}

impl<T> Dummy<T> for Company {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &T, rng: &mut R) -> Self {
        let fake_time =
            fake::faker::time::en::DateTime().fake_with_rng::<time::PrimitiveDateTime, R>(rng);

        let new_company = NewCompany::dummy_with_rng(_config, rng);

        Company {
            company_id: new_company.company_id,
            representative_name: new_company.representative_name,
            headquarters_address: new_company.headquarters_address,
            business_registration_number: new_company.business_registration_number,
            company_name: new_company.company_name,
            industry_code: new_company.industry_code,
            industry_name: new_company.industry_name,
            created_at: fake_time,
            updated_at: fake_time,
        }
    }
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = crate::schema::smes::company)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewCompany {
    pub company_id: model::company::Id,
    pub representative_name: String,
    pub headquarters_address: String,
    pub business_registration_number: String,
    pub company_name: String,
    pub industry_code: String,
    pub industry_name: String,
}

impl<T> Dummy<T> for NewCompany {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &T, rng: &mut R) -> Self {
        NewCompany {
            company_id: NumberWithFormat(EN, "^######")
                .fake::<String>()
                .as_str()
                .try_into()
                .expect("failed to create dummy company_id"),
            representative_name: Name().fake_with_rng::<String, R>(rng),
            headquarters_address: format!(
                "{}, South Korea",
                CityName().fake_with_rng::<String, R>(rng)
            ),
            business_registration_number: NumberWithFormat(EN, "^#########").fake::<String>(),
            company_name: CompanyName().fake_with_rng::<String, R>(rng),
            industry_code: NumberWithFormat(EN, "^####").fake::<String>(),
            industry_name: Industry().fake_with_rng::<String, R>(rng),
        }
    }
}

impl From<Company> for NewCompany {
    fn from(company: Company) -> Self {
        NewCompany {
            company_id: company.company_id,
            representative_name: company.representative_name,
            headquarters_address: company.headquarters_address,
            business_registration_number: company.business_registration_number,
            company_name: company.company_name,
            industry_code: company.industry_code,
            industry_name: company.industry_name,
        }
    }
}

impl PartialEq for NewCompany {
    fn eq(&self, other: &Self) -> bool {
        self.company_id == other.company_id
            && self.representative_name == other.representative_name
            && self.headquarters_address == other.headquarters_address
            && self.business_registration_number == other.business_registration_number
            && self.company_name == other.company_name
            && self.industry_code == other.industry_code
            && self.industry_name == other.industry_name
    }
}

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::smes::html)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Html {
    pub company_id: String,
    pub html_raw: String,
    pub created_at: time::PrimitiveDateTime,
    pub updated_at: time::PrimitiveDateTime,
}

impl<T> Dummy<T> for Html {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &T, rng: &mut R) -> Self {
        let fake_time =
            fake::faker::time::en::DateTime().fake_with_rng::<time::PrimitiveDateTime, R>(rng);
        let new_html = NewHtml::dummy_with_rng(_config, rng);

        Html {
            company_id: new_html.company_id,
            html_raw: new_html.html_raw,
            created_at: fake_time,
            updated_at: fake_time,
        }
    }
}

#[derive(Insertable, Clone)]
#[diesel(table_name = crate::schema::smes::html)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewHtml {
    pub company_id: String,
    pub html_raw: String,
}
impl<T> Dummy<T> for NewHtml {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &T, rng: &mut R) -> Self {
        NewHtml {
            company_id: NumberWithFormat(EN, "^######")
                .fake::<String>(),
            html_raw: format!(
                "<html><head><title>{}</title></head><body><h2>유동자산</h2><p>{}</p></body></html>",
                CompanyName().fake_with_rng::<String, R>(rng),
                Name().fake_with_rng::<String, R>(rng)
            )
        }
    }
}

impl From<Html> for NewHtml {
    fn from(html: Html) -> Self {
        NewHtml {
            company_id: html.company_id,
            html_raw: html.html_raw,
        }
    }
}
