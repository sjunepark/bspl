use diesel::{Insertable, Queryable, Selectable};
use fake::faker::address::ja_jp::CityName;
use fake::faker::company::ja_jp::{CompanyName, Industry};
use fake::faker::name::ja_jp::Name;
use fake::faker::number::raw::NumberWithFormat;
use fake::locales::EN;
use fake::{Dummy, Fake};
use rand::Rng;
use types::company;

// region: Table company
#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::smes::company)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Company {
    pub smes_id: company::SmesId,
    pub representative_name: company::RepresentativeName,
    pub headquarters_address: company::HeadquartersAddress,
    pub business_registration_number: company::BusinessRegistrationNumber,
    pub company_name: company::CompanyName,
    pub industry_code: company::IndustryCode,
    pub industry_name: company::IndustryName,
    pub created_at: time::PrimitiveDateTime,
    pub updated_at: time::PrimitiveDateTime,
}

impl<T> Dummy<T> for Company {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &T, rng: &mut R) -> Self {
        let fake_time =
            fake::faker::time::en::DateTime().fake_with_rng::<time::PrimitiveDateTime, R>(rng);

        let new_company = NewCompany::dummy_with_rng(_config, rng);

        Company {
            smes_id: new_company.smes_id,
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
    pub smes_id: company::SmesId,
    pub representative_name: company::RepresentativeName,
    pub headquarters_address: company::HeadquartersAddress,
    pub business_registration_number: company::BusinessRegistrationNumber,
    pub company_name: company::CompanyName,
    pub industry_code: company::IndustryCode,
    pub industry_name: company::IndustryName,
}

impl<T> Dummy<T> for NewCompany {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &T, rng: &mut R) -> Self {
        NewCompany {
            smes_id: NumberWithFormat(EN, "^######")
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
            company_name: CompanyName().fake_with_rng::<String, R>(rng).into(),
            industry_code: NumberWithFormat(EN, "^####")
                .fake::<String>()
                .as_str()
                .try_into()
                .expect("dummy creation logic needs to be fixed within the source code"),
            industry_name: Industry().fake_with_rng::<String, R>(rng).into(),
        }
    }
}

impl From<Company> for NewCompany {
    fn from(company: Company) -> Self {
        NewCompany {
            smes_id: company.smes_id,
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
        self.smes_id == other.smes_id
            && self.representative_name == other.representative_name
            && self.headquarters_address == other.headquarters_address
            && self.business_registration_number == other.business_registration_number
            && self.company_name == other.company_name
            && self.industry_code == other.industry_code
            && self.industry_name == other.industry_name
    }
}
// endregion: Table company

// region: Table html
#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::smes::html)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Html {
    pub smes_id: company::SmesId,
    pub html_content: company::SmesHtmlContent,
    pub created_at: time::PrimitiveDateTime,
    pub updated_at: time::PrimitiveDateTime,
}

impl<T> Dummy<T> for Html {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &T, rng: &mut R) -> Self {
        let fake_time =
            fake::faker::time::en::DateTime().fake_with_rng::<time::PrimitiveDateTime, R>(rng);
        let new_html = NewHtml::dummy_with_rng(_config, rng);

        Html {
            smes_id: new_html.smes_id,
            html_content: new_html.html_content,
            created_at: fake_time,
            updated_at: fake_time,
        }
    }
}

#[derive(Insertable, Clone, PartialEq, Debug)]
#[diesel(table_name = crate::schema::smes::html)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewHtml {
    pub smes_id: company::SmesId,
    pub html_content: company::SmesHtmlContent,
}
impl<T> Dummy<T> for NewHtml {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &T, rng: &mut R) -> Self {
        NewHtml {
            smes_id: NumberWithFormat(EN, "^######")
                .fake::<String>().as_str().try_into().expect("dummy creation logic needs to be fixed within the source code"),
            html_content: format!(
                "<html><head><title>{}</title></head><body><h2>유동자산</h2><p>{}</p></body></html>",
                CompanyName().fake_with_rng::<String, R>(rng),
                Name().fake_with_rng::<String, R>(rng)
            ).as_str().try_into().expect("dummy creation logic needs to be fixed within the source code"),
        }
    }
}

impl From<Html> for NewHtml {
    fn from(html: Html) -> Self {
        NewHtml {
            smes_id: html.smes_id,
            html_content: html.html_content,
        }
    }
}
// endregion: Table html
