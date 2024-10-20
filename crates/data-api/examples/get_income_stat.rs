use data_api::{DataApi, IncomeStatParamsBuilder};
use types::company::CorporationRegistrationNumber;

#[tokio::main]
async fn main() {
    tracing_setup::span!("example");

    let allow_external_api_call = std::env::var("GOLDRUST_ALLOW_EXTERNAL_API_CALL")
        .expect("GOLDRUST_ALLOW_EXTERNAL_API_CALL is not set");

    let allow_external_api_call = allow_external_api_call
        .parse::<bool>()
        .inspect_err(|e| {
            tracing::error!(
                ?allow_external_api_call,
                ?e,
                "Failed to parse GOLDRUST_ALLOW_EXTERNAL_API_CALL"
            );
        })
        .unwrap();

    if !allow_external_api_call {
        tracing::warn!("External API calls are disabled");
        return;
    }

    let api = DataApi::default();

    let params = IncomeStatParamsBuilder::default()
        .num_of_rows(100_u64)
        .page_no(1_u64)
        .crno(CorporationRegistrationNumber::try_new("1701110006868").unwrap())
        .biz_year("2023")
        .build()
        .expect("Failed to build params");

    let response = api
        .get_income_stat(params)
        .await
        .expect("Failed to get response");

    let text = response.text().await.expect("Failed to parse JSON");

    tracing::trace!(?text, "Received response");
}
