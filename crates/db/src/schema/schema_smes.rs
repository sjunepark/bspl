// @generated automatically by Diesel CLI.

pub mod smes {
    diesel::table! {
        smes.company (smes_id) {
            smes_id -> Text,
            representative_name -> Text,
            headquarters_address -> Text,
            business_registration_number -> Text,
            company_name -> Text,
            industry_code -> Text,
            industry_name -> Text,
            created_at -> Timestamp,
            updated_at -> Timestamp,
        }
    }

    diesel::table! {
        smes.html (smes_id) {
            smes_id -> Text,
            html_content -> Text,
            created_at -> Timestamp,
            updated_at -> Timestamp,
        }
    }

    diesel::joinable!(html -> company (smes_id));

    diesel::allow_tables_to_appear_in_same_query!(company, html,);
}
