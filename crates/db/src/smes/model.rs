use crate::DbError;
use chrono::{NaiveDate, Utc};
use chrono_tz::Asia;
use libsql::params::IntoParams;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use validator::{Validate, ValidationError};

pub trait Params {
    fn params(&self) -> impl IntoParams;
}

/// Represents a company with its details.
///
/// Example:
/// ```rust
/// use db::Company;
///
/// let company = Company {
///   id: String::from("1071180"),
///   representative_name: String::from("김성국"),
///   headquarters_address: String::from("경기도 김포시"),
///   business_registration_number: String::from("5632000760"),
///   company_name: String::from("루키게임즈"),
///   industry_code: String::from("63999"),
///   industry_name: String::from("그 외 기타 정보 서비스업"),
///   create_date: chrono::NaiveDate::from_ymd_opt(2024, 8, 1).unwrap(),
///   update_date: chrono::NaiveDate::from_ymd_opt(2024, 9, 30).unwrap(),
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct Company {
    /// 고유번호 (Unique Number)
    #[validate(length(min = 7, max = 7))]
    pub id: String,
    /// 대표자명 (Representative Name)
    pub representative_name: String,
    /// 본사주소 (Headquarters Address)
    pub headquarters_address: String,
    /// 사업자번호 (Business Registration Number)
    #[validate(custom(function = "length_10_or_empty"))]
    pub business_registration_number: String,
    /// 기업명 (Company Name)
    pub company_name: String,
    /// 업종코드 (Industry Code)
    #[validate(length(min = 5, max = 5))]
    pub industry_code: String,
    /// 업종 (Industry Name)
    pub industry_name: String,
    pub create_date: NaiveDate,
    pub update_date: NaiveDate,
}

fn length_10_or_empty(value: &str) -> Result<(), validator::ValidationError> {
    if value.is_empty() || value.len() == 10 {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_length"))
    }
}

impl TryFrom<smes::Company> for Company {
    type Error = DbError;

    fn try_from(value: smes::Company) -> Result<Self, Self::Error> {
        let now = Utc::now().with_timezone(&Asia::Seoul).date_naive();

        let company = Self {
            id: value.vnia_sn.to_string(),
            representative_name: value.rprsv_nm,
            headquarters_address: value.hdofc_addr,
            business_registration_number: value.bizrno,
            company_name: value.cmp_nm,
            industry_code: value.indsty_cd,
            industry_name: value.indsty_nm,
            create_date: now,
            update_date: now,
        };
        company.validate()?;
        Ok(company)
    }
}

impl Params for Company {
    fn params(&self) -> impl IntoParams {
        libsql::named_params! {
            ":id": self.id.as_str(),
            ":representative_name": self.representative_name.as_str(),
            ":headquarters_address": self.headquarters_address.as_str(),
            ":business_registration_number": self.business_registration_number.as_str(),
            ":company_name": self.company_name.as_str(),
            ":industry_code": self.industry_code.as_str(),
            ":industry_name": self.industry_name.as_str(),
            ":create_date": self.create_date.to_string(),
            ":update_date": self.update_date.to_string(),
        }
    }
}

pub struct Companies(Vec<Company>);

impl TryFrom<Vec<smes::Company>> for Companies {
    type Error = DbError;

    fn try_from(value: Vec<smes::Company>) -> Result<Self, Self::Error> {
        let len = value.len();
        let companies = value
            .into_iter()
            .try_fold(Vec::with_capacity(len), |mut acc, c| {
                let company = Company::try_from(c)?;
                acc.push(company);
                Ok::<Vec<Company>, DbError>(acc)
            })?;
        Ok(Self(companies))
    }
}

impl Deref for Companies {
    type Target = Vec<Company>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// This implementation is necessary to create fake `Company` structs for tests,
// such as `().fake::<Company>().`
#[cfg(test)]
mod test_impl {
    use super::*;
    use chrono::Local;
    use fake::faker::address::ja_jp::CityName;
    use fake::faker::company::ja_jp::{CompanyName, Industry};
    use fake::faker::name::ja_jp::Name;
    use fake::faker::number::raw::NumberWithFormat;
    use fake::locales::EN;
    use fake::{Dummy, Fake};
    use rand::Rng;

    impl<T> Dummy<T> for Company {
        fn dummy_with_rng<R: Rng + ?Sized>(_config: &T, rng: &mut R) -> Self {
            Company {
                id: NumberWithFormat(EN, "^#########")
                    .fake::<String>()
                    .parse()
                    .inspect_err(|e| tracing::error!(?e, "Failed to parse number"))
                    .unwrap(),
                representative_name: Name().fake_with_rng(rng),
                headquarters_address: format!(
                    "{}, South Korea",
                    CityName().fake_with_rng::<String, R>(rng)
                ),
                business_registration_number: NumberWithFormat(EN, "^#########").fake::<String>(),
                company_name: CompanyName().fake_with_rng(rng),
                industry_code: NumberWithFormat(EN, "^####").fake::<String>(),
                industry_name: Industry().fake_with_rng(rng),
                create_date: Local::now().naive_local().date(),
                update_date: Local::now().naive_local().date(),
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn system_should_return_proper_utc_date() {
        tracing_setup::subscribe();
        let utc = chrono::Utc::now();
        let local = chrono::Local::now();
        let naive_utc = utc.naive_local();
        let naive_local = local.naive_local();
        tracing::debug!(?utc, ?naive_utc, ?local, ?naive_local);

        let duration_in_hours = (naive_local - naive_utc).num_hours();
        assert_eq!(duration_in_hours, 9);
    }
}
