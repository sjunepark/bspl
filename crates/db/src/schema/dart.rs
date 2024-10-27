// @generated automatically by Diesel CLI.

pub mod dart {
    diesel::table! {
        dart.filing (corp_code) {
            corp_code -> Text,
            corp_name -> Text,
            stock_code -> Text,
            corp_cls -> Text,
            report_nm -> Text,
            rcept_no -> Text,
            flr_nm -> Text,
            rcept_dt -> Date,
            rm -> Text,
            created_at -> Timestamp,
            updated_at -> Timestamp,
        }
    }
}
