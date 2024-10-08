use crate::api::model::{Captcha, Solved, Submitted, Unsubmitted};
use crate::api::NopechaApi;
use crate::error::NopechaError;
use crate::{BsplApi, SmesError};
use std::time::Duration;
use tokio::sync::mpsc::{channel, unbounded_channel, Receiver, UnboundedReceiver};
use tracing::Instrument;

/// Get solved captchas with answers. These can be used to query bspls.
/// * `count` - The number of captchas to fetch.
///             This will correspond to the number of companies you want to query.
#[tracing::instrument]
pub async fn get_solved_captchas(count: usize) -> UnboundedReceiver<Captcha<Solved>> {
    const BUFFER_SIZE: usize = 8;
    let unsubmitted_captchas = get_captchas(count, BUFFER_SIZE).await;
    let submitted_captchas = submit_captchas(unsubmitted_captchas).await;
    get_answers(submitted_captchas).await
}

/// Fetch many captchas to solve.
///
/// * `count` - The number of captchas to fetch.
///             This will correspond to the number of companies you want to query.
/// * `cap` - The maximum number of captchas to buffer in the channel.
///           This is necessary so an infinite number of captcha requests to smes does not happen.
///
/// This function sends captchas to a channel that the receiver can consume.
/// It does not introduce any artificial delays between requests;
/// therefore, it may aggressively request captchas from the server if the receiver processes them quickly.
/// The receiver should control the rate of captcha processing to avoid overwhelming the server.
///
/// ## Warning
/// The function is in an infinite loop until the receiver closes the channel.
/// Make sure to close the receiver.
#[tracing::instrument]
async fn get_captchas(count: usize, cap: usize) -> Receiver<Captcha<Unsubmitted>> {
    let (tx, rx) = channel::<Captcha<Unsubmitted>>(cap);
    let mut api = BsplApi::default();

    tokio::spawn(
        async move {
            for _ in 0..count {
                let tx = tx.clone();
                let captcha = match api.get_captcha().await {
                    Ok(captcha) => captcha,
                    Err(e) => {
                        tracing::warn!(?e, "Failed to get captcha. Skipping.");
                        continue;
                    }
                };
                if let Err(e) = tx.send(captcha).await {
                    tracing::warn!(
                        ?e,
                        "Failed to send captcha. The channel has been closed. Skipping."
                    );
                    break;
                }
            }
        }
        .in_current_span(),
    );

    rx
}

/// Submit captchas to the Nopecha API to get answers.
///
/// * `captchas` - A channel receiver of unsubmitted captchas to solve
///
/// The function will return only the captchas without errors.
/// The errors will be logged (WARN) and discarded.
///
/// This function sends captchas to a channel that the receiver can consume.
/// It does not introduce any artificial delays between requests;
/// therefore, it may aggressively request captchas from the server if the receiver processes them quickly.
/// The receiver should control the rate of captcha processing to avoid overwhelming the server.
#[tracing::instrument(skip(captchas))]
async fn submit_captchas(
    mut captchas: Receiver<Captcha<Unsubmitted>>,
) -> UnboundedReceiver<Captcha<Submitted>> {
    let (tx, rx) = unbounded_channel::<Captcha<Submitted>>();
    let api = NopechaApi::default();

    tokio::spawn(
        async move {
            while let Some(captcha) = captchas.recv().await {
                match api.submit_captcha(captcha).await {
                    Ok(captcha) => {
                        tx.send(captcha).unwrap_or_else(|e| {
                            tracing::warn!(
                                ?e,
                                "Failed to send captcha. The channel has been closed. Skipping."
                            );
                        });
                    }
                    Err(SmesError::Nopecha(NopechaError::OutOfCredit(e))) => {
                        tracing::warn!(?e, "Nopecha API out of credit. Stopping.");
                        break;
                    }
                    Err(e) => {
                        tracing::warn!(?e, "Error received from submit_captcha. Skipping.");
                    }
                }
            }
        }
        .in_current_span(),
    );

    rx
}

#[tracing::instrument(skip(captchas))]
async fn get_answers(
    mut captchas: UnboundedReceiver<Captcha<Submitted>>,
) -> UnboundedReceiver<Captcha<Solved>> {
    let (tx, rx) = unbounded_channel::<Captcha<Solved>>();

    tokio::spawn(
        async move {
            let api = NopechaApi::default();

            while let Some(captcha) = captchas.recv().await {
                match api
                    .get_answer_with_retries(&captcha, 10, Duration::from_secs(1))
                    .await
                {
                    Ok(captcha) => {
                        tx.send(captcha).unwrap_or_else(|e| {
                            tracing::warn!(
                                ?e,
                                "Failed to send captcha. The channel has been closed. Skipping."
                            );
                        });
                    }
                    Err(e) => match e {
                        SmesError::Nopecha(NopechaError::OutOfCredit(e)) => {
                            tracing::warn!(?e, "Nopecha API out of credit. Stopping.");
                            break;
                        }
                        _ => {
                            tracing::warn!(
                                ?e,
                                "Error received while running get_answer_with_retries. Skipping."
                            );
                        }
                    },
                }
            }
        }
        .in_current_span(),
    );

    rx
}
