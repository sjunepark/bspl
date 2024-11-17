use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};
use types::company;

#[cfg(test)]
use fake::faker::{
    address::ja_jp::CityName,
    company::ja_jp::{CompanyName, Industry},
    name::en::Name,
    number::raw::NumberWithFormat,
};
#[cfg(test)]
use fake::locales::EN;
#[cfg(test)]
use fake::{Dummy, Fake, Faker};

// region: Table company
#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::smes::company)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(test, derive(fake::Dummy))]
pub struct Company {
    pub smes_id: company::SmesId,
    pub representative_name: company::RepresentativeName,
    pub headquarters_address: company::HeadquartersAddress,
    pub business_registration_number: company::BusinessRegistrationNumber,
    pub company_name: company::CompanyName,
    pub industry_code: company::IndustryCode,
    pub industry_name: company::IndustryName,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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

#[cfg(test)]
impl Dummy<Faker> for NewCompany {
    fn dummy_with_rng<R: rand::Rng + ?Sized>(_config: &Faker, rng: &mut R) -> Self {
        NewCompany {
            smes_id: NumberWithFormat(EN, "^######")
                .fake::<String>()
                .as_str()
                .try_into()
                .expect("dummy creation logic needs to be fixed within the source code"),
            representative_name: Name().fake_with_rng::<String, R>(rng),
            headquarters_address: format!(
                "{}, South Korea",
                CityName().fake_with_rng::<String, R>(rng)
            ),
            business_registration_number: NumberWithFormat(EN, "^#########")
                .fake::<String>()
                .as_str()
                .try_into()
                .expect("dummy creation logic needs to be fixed within the source code"),
            company_name: CompanyName().fake_with_rng::<String, R>(rng),
            industry_code: NumberWithFormat(EN, "^####")
                .fake::<String>()
                .as_str()
                .try_into()
                .expect("dummy creation logic needs to be fixed within the source code"),
            industry_name: Industry().fake_with_rng::<String, R>(rng),
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
#[cfg_attr(test, derive(fake::Dummy))]
pub struct Html {
    pub smes_id: company::SmesId,
    pub html_content: company::SmesHtmlContent,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Clone, PartialEq, Debug)]
#[diesel(table_name = crate::schema::smes::html)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewHtml {
    pub smes_id: company::SmesId,
    pub html_content: company::SmesHtmlContent,
}

#[cfg(test)]
impl Dummy<Faker> for NewHtml {
    fn dummy_with_rng<R: rand::Rng + ?Sized>(_config: &Faker, rng: &mut R) -> Self {
        NewHtml {
            smes_id: NumberWithFormat(EN, "^######")
                .fake::<String>().as_str().try_into().expect("dummy creation logic needs to be fixed within the source code"),
            html_content: format!(
                "<html><head><title>{}</title></head><body><h2>유동자산</h2><p>{}</p></body></html>",
                CompanyName().fake_with_rng::<String, R>(rng),
                Name().fake_with_rng::<String, R>(rng)
            ),
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
