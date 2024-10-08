use crate::BsplApi;
use model::table;
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
pub async fn get_bspl_htmls(companies: Vec<model::company::Id>) -> UnboundedReceiver<table::Html> {
    let (tx, rx) = unbounded_channel::<table::Html>();
    let size = companies.len();
    let mut captcha_cookies = captcha::get_solved_captchas(size).await;
    let mut ids = companies.into_iter();

    tokio::spawn(
        async move {
            let mut api = BsplApi::default();
            let mut index = 0;

            while let Some(captcha) = captcha_cookies.recv().await {
                let Some(id) = ids.next() else { continue };
                tracing::trace!("Getting {}/{} company's bspl html", index + 1, size);

                let html = match api
                    .get_bspl_html(captcha.cookies(), id.as_str(), captcha.answer())
                    .await
                {
                    Ok(html) => html,
                    Err(e) => {
                        tracing::warn!(?e, "Error received from get_bspl_html. Skipping.");
                        continue;
                    }
                };

                let html = crate::Html {
                    vnia_sn: id.to_string(),
                    html: html.into(),
                }
                .try_into();

                let html = match html {
                    Ok(html) => html,
                    Err(e) => {
                        tracing::warn!(?e, "Error converting html to db::Html. Skipping.");
                        continue;
                    }
                };

                match tx.send(html) {
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
        }
        .in_current_span(),
    );

    rx
}
