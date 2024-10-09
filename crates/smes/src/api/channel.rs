use crate::BsplApi;
use hashbrown::HashSet;
use model::{company, table};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tracing::Instrument;

mod captcha;

/// The entry point of getting the bspl HTMLs.
///
/// * `companies` - A collection of company ids
///
/// This function will perform multiple operations, communicating with channels.
/// 1. Get captcha images to solve
/// 2. Solve the captchas and store each answer with the corresponding captcha
/// 3. Request for the corresponding bspl HTML with the captcha answer
///
/// Process #1 and #2 are performed by `get_captcha_cookies`.
///
/// ## Error handling
/// When an error occurs during the process,
/// the error will be logged and the process will continue,
/// skipping the corresponding operation.
///
/// The skipped operations should be inspected and re-scraped in the future if necessary.
#[tracing::instrument(skip(companies))]
pub async fn get_bspl_htmls(companies: HashSet<company::Id>) -> UnboundedReceiver<table::Html> {
    let (tx, rx) = unbounded_channel::<table::Html>();
    let size = companies.len();
    let mut captcha_cookies = captcha::get_solved_captchas(size).await;
    let ids = companies.into_iter();

    tokio::spawn(
        async move {
            let mut api = BsplApi::default();

            const MAX_RETRY_PER_ID: usize = 3;

            'id: for (index, id) in ids.enumerate() {
                'retry: for retry in 0..=MAX_RETRY_PER_ID {
                    match captcha_cookies.recv().await {
                        None => {
                            tracing::warn!("Out of captchas. End of loop.");
                        }
                        Some(captcha) => {
                            let span = tracing::info_span!("get_bspl_html", ?id);
                            let _guard = span.enter();
                            if retry > 0 {
                                tracing::warn!(
                                    ?id,
                                    "Retrying {}/{} get_bspl_html with new captcha",
                                    retry + 1,
                                    MAX_RETRY_PER_ID
                                );
                            }

                            tracing::info!("Getting {}/{} company's bspl html", index + 1, size);

                            // Run `get_bspl_html` but continue with loop when it errors.
                            // This is because the errors from `get_bspl_html`
                            // are not considered to be recoverable through retries.
                            let html = match api
                                .get_bspl_html(id.as_str(), &captcha)
                                .in_current_span()
                                .await
                            {
                                Ok(html) => html,
                                Err(e) => {
                                    tracing::warn!(
                                        ?e,
                                        ?id,
                                        "Error received from get_bspl_html. Skipping id."
                                    );
                                    continue 'id;
                                }
                            };

                            // Try converting the String HTML to db::Html.
                            // This will fail when the HTML does not contain the required string "유동자산".
                            let html = crate::Html {
                                vnia_sn: id.to_string(),
                                html,
                            }
                            .try_into();

                            // Retry when the HTML does not contain the required string "유동자산".
                            let html = match html {
                                Ok(html) => html,
                                Err(e) => {
                                    tracing::warn!(?e, ?id, "Error converting html to db::Html.");
                                    continue 'retry;
                                }
                            };

                            if let Err(e) = tx.send(html) {
                                tracing::warn!(?e, ?id, "Failed to send bspl html. The channel has been closed. Breaking loop.");
                                break 'id;
                            }
                        }
                    }
                }
            }
        }
        .in_current_span(),
    );

    rx
}
