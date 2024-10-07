use crate::api::model::BsPl;
use crate::{BsplApi, VniaSn};
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
pub async fn get_bspl_htmls(companies: &[VniaSn]) -> UnboundedReceiver<BsPl> {
    let (tx, rx) = unbounded_channel::<BsPl>();
    let captcha_count = companies.len();
    let mut captcha_cookies = captcha::get_solved_captchas(captcha_count).await;

    let companies_count = companies.len();
    let companies = companies.to_owned();

    tokio::spawn(
        async move {
            let mut api = BsplApi::default();
            let mut index = 0;

            while let Some(captcha) = captcha_cookies.recv().await {
                if index >= companies_count {
                    break;
                }

                tracing::trace!(
                    "Getting {}/{} company's bspl html",
                    index + 1,
                    companies_count
                );
                let vnia_sn: VniaSn = companies[index];
                let html = api
                    .get_bspl_html(captcha.cookies(), *vnia_sn, captcha.answer())
                    .await;

                match html {
                    Ok(html) => {
                        match tx.send(BsPl { vnia_sn, html }) {
                            Ok(_) => {
                                index += 1;
                            }
                            Err(e) => {
                                tracing::warn!(
                                ?e,
                                "Failed to send bspl html. The channel has been closed. Skipping."
                            );
                            }
                        };
                    }
                    Err(e) => {
                        tracing::warn!(?e, "Error received from get_bspl_html. Skipping.");
                    }
                }
            }
        }
        .in_current_span(),
    );

    rx
}
