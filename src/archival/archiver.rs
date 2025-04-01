use async_fn_stream::try_fn_stream;
use futures::pin_mut;
use futures::Stream;
use futures::StreamExt;
use futures::TryStreamExt;
use log::{error, info, warn};
use sqlx::PgPool;
use streamies::TryStreamies;
use tokio::time;

use crate::api::internet_archive::archive_url_in_ia;
use crate::configuration::SETTINGS;
use crate::metrics::Metrics;
use crate::models::melba::internet_archive_urls::InternetArchiveUrl;

/// Run the archiver task
pub async fn run_archiver_task(conn: &PgPool) -> Result<(), crate::Error> {
    let url_stream = get_archiver_stream(conn.clone());

    let url_stream = url_stream
        .map_ok(|url| process_url(conn, url))
        .extract_future_ok()
        .buffer_unordered(SETTINGS.archival_task.worker_count as usize)
        .flatten_result_ok();

    pin_mut!(url_stream);

    // Run the archiver
    while let Some(_t) = url_stream.try_next().await? {}

    Ok(())
}

/// Create a stream of urls to archive
fn get_archiver_stream(
    conn: PgPool,
) -> impl Stream<Item = Result<InternetArchiveUrl, crate::Error>> {
    try_fn_stream(async move |emitter| {
        let mut interval = time::interval(SETTINGS.archival_task.get_job_interval());

        loop {
            let res = InternetArchiveUrl::find_new_job(&conn, None).await?;

            if let Some(mut url) = res {
                url.set_processing(&conn).await?;
                emitter.emit(url).await;
            }

            interval.tick().await;
        }
    })
}

/// Process an url element
async fn process_url(conn: &PgPool, mut url: InternetArchiveUrl) -> Result<(), crate::Error> {
    let metrics = Metrics::new().await;
    info!("[Archiver] Processing url id `{}`", url.id);

    // We check if the url hasn't been retried multiple times already
    if (url.try_count as i64) < SETTINGS.archival_task.max_retry {
        request_archiving(conn, &mut url).await?;
    } else {
        warn!(
            "[Archiver] Too many retries for url: id {}, URL: {}, reason: {}",
            url.id,
            url.url,
            url.status_message.as_ref().unwrap_or(&"(None)".to_string())
        );
    }

    Ok(())
}

/// Request the url archival for IA. Save the job id if successful
async fn request_archiving(
    conn: &PgPool,
    url: &mut InternetArchiveUrl,
) -> Result<(), crate::Error> {
    match archive_url_in_ia(&url.url).await {
        // IA returned the job id. Set the url as waiting status, and let the status checker task handle it
        Ok(response) => {
            info!(
                "[Archiver] Request successful | url id: {} url: {}, job id: {}",
                url.id, url.url, response.job_id
            );
            url.set_waiting_status(conn, response.job_id).await?;
        }

        // Uh oh, error. Set the url as errored,
        Err(err) => {
            error!(
                "[Archiver] Request error | url id: {} url: {}, error: {}",
                url.id, url.url, err
            );

            //TODO: Check whether the error is from this specific URL (Set as errored), or something external (return the error to be dealt higher up).
            url.set_errored(conn).await?;
        }
    }

    Ok(())
}
