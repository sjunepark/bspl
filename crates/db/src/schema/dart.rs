// @generated automatically by Diesel CLI.

pub mod dart {
    diesel::table! {
        dart.filing (dart_id) {
            dart_id -> Text,
            report_name -> Text,
            receipt_number -> Text,
            filer_name -> Text,
            receipt_date -> Date,
            remark -> Text,
            created_at -> Timestamp,
            updated_at -> Timestamp,
        }
    }
}
