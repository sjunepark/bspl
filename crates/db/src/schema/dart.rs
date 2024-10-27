// @generated automatically by Diesel CLI.

pub mod dart {
    diesel::table! {
        dart.filing (corp_code) {
            corp_code -> Text,
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
