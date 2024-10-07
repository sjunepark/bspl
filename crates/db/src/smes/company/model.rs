use crate::smes::utils::length_10_or_empty;
use crate::DbError;
use chrono::{NaiveDate, Utc};
use chrono_tz::Asia;
use libsql::params::IntoParams;
use serde::{Deserialize, Serialize};
use validator::Validate;

pub trait Params {
    fn params(&self) -> impl IntoParams;
}

// todo: Maybe remove validation if `model` insures everything is initialized correctly.
/// Represents a company with its details.
///
/// Example:
/// ```rust
/// use db::Company;
///
/// let company = Company {
///   smes_id: String::from("1071180"),
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
    pub smes_id: String,
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

impl TryFrom<smes::Company> for Company {
    type Error = DbError;

    fn try_from(value: smes::Company) -> Result<Self, Self::Error> {
        let now = Utc::now().with_timezone(&Asia::Seoul).date_naive();

        let company = Self {
            smes_id: value.vnia_sn.to_string(),
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
            ":smes_id": self.smes_id.as_str(),
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

// This implementation is necessary to create fake `Company` structs for tests,
// such as `().fake::<Company>().`
#[cfg(test)]
mod test_impl {
    use super::*;
    use fake::faker::address::ja_jp::CityName;
    use fake::faker::company::ja_jp::{CompanyName, Industry};
    use fake::faker::name::ja_jp::Name;
    use fake::faker::number::raw::NumberWithFormat;
    use fake::locales::EN;
    use fake::{Dummy, Fake};
    use rand::Rng;

    impl<T> Dummy<T> for Company {
        fn dummy_with_rng<R: Rng + ?Sized>(_config: &T, rng: &mut R) -> Self {
            let now = Utc::now().with_timezone(&Asia::Seoul).date_naive();

            Company {
                smes_id: NumberWithFormat(EN, "^######").fake::<String>(),
                representative_name: Name().fake_with_rng(rng),
                headquarters_address: format!(
                    "{}, South Korea",
                    CityName().fake_with_rng::<String, R>(rng)
                ),
                business_registration_number: NumberWithFormat(EN, "^#########").fake::<String>(),
                company_name: CompanyName().fake_with_rng(rng),
                industry_code: NumberWithFormat(EN, "^####").fake::<String>(),
                industry_name: Industry().fake_with_rng(rng),
                create_date: now,
                update_date: now,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smes::VniaSn;

    #[test]
    fn company_try_from_should_fail_for_invalid_field_values() {
        let company = smes::Company {
            vnia_sn: VniaSn(1071180),
            rprsv_nm: "김성국".to_string(),
            hdofc_addr: "경기도 김포시".to_string(),
            bizrno: "5632000760".to_string(),
            cmp_nm: "루키게임즈".to_string(),
            indsty_cd: "63999".to_string(),
            indsty_nm: "그 외 기타 정보 서비스업".to_string(),
        };

        let invalid_id = VniaSn(123456);
        let invalid_business_registration_number = "123456789";
        let invalid_industry_code = "1234";

        let invalid_id_company = smes::Company {
            vnia_sn: invalid_id,
            ..company.clone()
        };
        assert!(Company::try_from(invalid_id_company).is_err());

        let invalid_business_registration_number_company = smes::Company {
            bizrno: invalid_business_registration_number.to_string(),
            ..company.clone()
        };
        assert!(Company::try_from(invalid_business_registration_number_company).is_err());

        let invalid_industry_code_company = smes::Company {
            indsty_cd: invalid_industry_code.to_string(),
            ..company
        };
        assert!(Company::try_from(invalid_industry_code_company).is_err());
    }
}
