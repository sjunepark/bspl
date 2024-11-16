use crate::DbError;
use chrono::NaiveDate;
use diesel::{Insertable, Queryable, Selectable};
use fake::faker::name::ja_jp::Name;
use fake::faker::number::raw::NumberWithFormat;
use fake::locales::EN;
use fake::{Dummy, Fake};
use rand::Rng;
use types::{company, filing, YYYYMMDD};

// region: Table filing

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::dart::filing)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Filing {
    pub dart_id: company::DartId,
    pub report_name: filing::ReportName,
    pub receipt_number: filing::ReceiptNumber,
    pub filer_name: filing::FilerName,
    pub receipt_date: filing::ReceiptDate,
    pub remark: filing::Remark,
    pub created_at: time::PrimitiveDateTime,
    pub updated_at: time::PrimitiveDateTime,
}

impl<T> Dummy<T> for Filing {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &T, rng: &mut R) -> Self {
        let fake_time =
            fake::faker::time::en::DateTime().fake_with_rng::<time::PrimitiveDateTime, R>(rng);

        let new_filing = NewFiling::dummy_with_rng(_config, rng);

        Filing {
            dart_id: new_filing.dart_id,
            report_name: new_filing.report_name,
            receipt_number: new_filing.receipt_number,
            filer_name: new_filing.filer_name,
            receipt_date: new_filing.receipt_date,
            remark: new_filing.remark,
            created_at: fake_time,
            updated_at: fake_time,
        }
    }
}

#[derive(Insertable, Clone, Debug)]
#[diesel(table_name = crate::schema::dart::filing)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewFiling {
    pub dart_id: company::DartId,
    pub report_name: filing::ReportName,
    pub receipt_number: filing::ReceiptNumber,
    pub filer_name: filing::FilerName,
    pub receipt_date: filing::ReceiptDate,
    pub remark: filing::Remark,
}

impl<T> Dummy<T> for NewFiling {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &T, rng: &mut R) -> Self {
        NewFiling {
            dart_id: NumberWithFormat(EN, "^#######")
                .fake::<String>()
                .as_str()
                .try_into()
                .expect("dummy creation logic needs to be fixed within the source code"),
            report_name: Name().fake_with_rng::<String, R>(rng),
            receipt_number: NumberWithFormat(EN, "^#############")
                .fake::<String>()
                .as_str()
                .try_into()
                .expect("dummy creation logic needs to be fixed within the source code"),
            filer_name: Name().fake_with_rng::<String, R>(rng),
            receipt_date: filing::ReceiptDate::new(
                NaiveDate::from_ymd_opt(2021, 1, 1).expect("invalid date passed"),
            ),
            remark: "Remark".into(),
        }
    }
}

impl From<Filing> for NewFiling {
    fn from(filing: Filing) -> Self {
        NewFiling {
            dart_id: filing.dart_id,
            report_name: filing.report_name,
            receipt_number: filing.receipt_number,
            filer_name: filing.filer_name,
            receipt_date: filing.receipt_date,
            remark: filing.remark,
        }
    }
}

impl PartialEq for NewFiling {
    fn eq(&self, other: &Self) -> bool {
        self.dart_id == other.dart_id
            && self.report_name == other.report_name
            && self.receipt_number == other.receipt_number
            && self.filer_name == other.filer_name
            && self.receipt_date == other.receipt_date
            && self.remark == other.remark
    }
}

// endregion: Table filing

// region: Table company_id

#[derive(Queryable, Selectable, Clone, Insertable, PartialEq, Debug)]
#[diesel(table_name = crate::schema::dart::company_id)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CompanyId {
    pub dart_id: company::DartId,
    pub company_name: company::CompanyName,
    pub stock_code: company::StockCode,
    pub id_modify_date: YYYYMMDD,
}

impl<T> Dummy<T> for CompanyId {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &T, rng: &mut R) -> Self {
        CompanyId {
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

impl TryFrom<crate::entities::dart::company_id::Model> for CompanyId {
    type Error = DbError;

    fn try_from(model: crate::entities::dart::company_id::Model) -> Result<Self, Self::Error> {
        Ok(CompanyId {
            dart_id: model.dart_id,
            company_name: model.company_name,
            stock_code: model.stock_code,
            id_modify_date: model.id_modify_date,
        })
    }
}

impl From<CompanyId> for crate::entities::dart::company_id::Model {
    fn from(company_id: CompanyId) -> Self {
        crate::entities::dart::company_id::Model {
            dart_id: company_id.dart_id,
            company_name: company_id.company_name,
            stock_code: company_id.stock_code,
            id_modify_date: company_id.id_modify_date,
        }
    }
}

// endregion: Table company_id
