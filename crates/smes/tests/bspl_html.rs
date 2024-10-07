use smes::api::get_bspl_htmls;
use smes::VniaSn;
use tracing::Instrument;

#[tokio::test]
async fn get_bspl_htmls_should_work_as_expected() {
    const TEST_COUNT: usize = 5;

    tracing_setup::span!("test");
    let test_id = utils::function_id!();
    let _span = tracing::info_span!("test", ?test_id).entered();

    tracing::info!("Starting test");

    let allow_external_api_call = std::env::var("GOLDRUST_ALLOW_EXTERNAL_API_CALL")
        .unwrap_or("false".to_string())
        .parse::<bool>()
        .expect("Failed to parse GOLDRUST_ALLOW_EXTERNAL_API_CALL to bool");

    if !allow_external_api_call {
        return;
    }

    let companies: Vec<VniaSn> = vec![
        1071180, 1104102, 1077757, 1049868, 1074520, 1112487, 1107135, 1074136, 1066081, 1113680,
        1062952, 1097842, 1118552, 1065357, 1122340, 1038994, 1063040, 1077914, 1124797, 1119565,
        1081050, 1082252, 1066341, 1116040, 1035895, 1117355, 1082766, 1057328, 1107294, 1072859,
    ]
    .iter()
    .take(TEST_COUNT)
    .map(|&id| VniaSn(id))
    .collect();

    let mut rx = get_bspl_htmls(&companies).in_current_span().await;

    let mut bspl_count = 0_usize;

    while let Some(bspl) = rx.recv().await {
        bspl_count += 1;
        let success = bspl.html.contains("유동자산");

        if !success {
            tracing::error!(?bspl, "Invalid html received");
        }

        assert!(success);
    }

    assert_eq!(bspl_count, TEST_COUNT);
}
