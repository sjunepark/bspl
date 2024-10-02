use libsql::params::IntoParams;
use serde::{Deserialize, Serialize};

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
///   industry_name: String::from("그 외 기타 정보 서비스업")
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Company {
    /// 고유번호 (Unique Number)
    pub id: String,
    /// 대표자명 (Representative Name)
    pub representative_name: String,
    /// 본사주소 (Headquarters Address)
    pub headquarters_address: String,
    /// 사업자번호 (Business Registration Number)
    pub business_registration_number: String,
    /// 기업명 (Company Name)
    pub company_name: String,
    /// 업종코드 (Industry Code)
    pub industry_code: String,
    /// 업종 (Industry Name)
    pub industry_name: String,
}

impl From<smes::Company> for Company {
    fn from(value: smes::Company) -> Self {
        Self {
            id: value.vnia_sn.to_string(),
            representative_name: value.rprsv_nm,
            headquarters_address: value.hdofc_addr,
            business_registration_number: value.bizrno,
            company_name: value.cmp_nm,
            industry_code: value.indsty_cd,
            industry_name: value.indsty_nm,
        }
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
            ":industry_name": self.industry_name.as_str()
        }
    }
}

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
            }
        }
    }
}
