// @generated automatically by Diesel CLI.

pub mod dart {
    diesel::table! {
        dart.company_all (company_id) {
            company_id -> Text,
            company_name -> Text,
            stock_code -> Text,
            modify_date -> Date,
        }
    }

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

    diesel::allow_tables_to_appear_in_same_query!(company_all, filing,);
}
