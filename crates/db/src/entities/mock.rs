#![cfg(test)]

use super::dart;
use chrono::NaiveDate;
use fake::faker::name::ja_jp::Name;
use fake::faker::number::raw::NumberWithFormat;
use fake::locales::EN;
use fake::{Dummy, Fake, Faker};
use rand::Rng;
use types::YYYYMMDD;

impl Dummy<Faker> for dart::company_id::Model {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &Faker, rng: &mut R) -> Self {
        Self {
            dart_id: NumberWithFormat(EN, "^#######")
                .fake::<String>()
                .as_str()
                .try_into()
                .expect("dummy creation logic needs to be fixed within the source code"),
            company_name: Name().fake_with_rng::<String, R>(rng),
            stock_code: NumberWithFormat(EN, "^#####")
                .fake::<String>()
                .as_str()
                .try_into()
                .expect("dummy creation logic needs to be fixed within the source code"),
            id_modify_date: YYYYMMDD::new(
                NaiveDate::from_ymd_opt(2021, 1, 1).expect("invalid date passed"),
            ),
        }
    }
}

impl Dummy<Faker> for dart::company_id::ActiveModel {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &Faker, rng: &mut R) -> Self {
        dart::company_id::Model::dummy_with_rng(_config, rng).into()
    }
}

// impl Dummy<Faker> for dart::filing::Model {
//     fn dummy_with_rng<R: Rng + ?Sized>(_config: &Faker, rng: &mut R) -> Self {
//         Self {
//             dart_id: Faker.fake::<DartId>(),
//             report_name: Name().fake_with_rng::<String, R>(rng),
//             receipt_number: Faker.fake::<ReceiptNumber>(),
//             filer_name: Name().fake_with_rng::<String, R>(rng),
//             receipt_date: NaiveDate::from_ymd_opt(2021, 1, 1)
//                 .expect("invalid date passed")
//                 .into(),
//             remark: "Remark".into(),
//             created_at: Faker.fake::<YYYYMMDD>().into(),
//             updated_at: Faker.fake::<YYYYMMDD>().into(),
//         }
//     }
// }
