use std::sync::Arc;
use std::sync::LazyLock;

use async_fn_stream::try_fn_stream;
use futures::pin_mut;
use futures::Stream;
use futures::StreamExt;
use futures::TryStreamExt;
use log::error;
use log::info;
use log::warn;
use sqlx::PgPool;
use streamies::TryStreamies as _;
use tokio::sync::Mutex;
use tokio::time;

use crate::api::internet_archive::archiving_job_status;
use crate::archival::archival_response::ArchivalStatusErrorResponse;
use crate::archival::archival_response::ArchivalStatusResponse;
use crate::archival::error::ArchivalError;
use crate::archival::utils::check_if_permanent_error;
use crate::configuration::SETTINGS;
use crate::models::melba::internet_archive_urls::InternetArchiveUrl;

pub async fn run_status_watcher(conn: &PgPool) -> Result<(), ArchivalError> {
    let url_stream = get_status_stream(conn.clone());

    let url_stream = url_stream
        .map_ok(|url| watch_status(conn, url))
        .extract_future_ok()
        .buffer_unordered(SETTINGS.status_watch_task.worker_count as usize)
        .flatten_result_ok();

    pin_mut!(url_stream);

    // Run the archiver
    while let Some(_t) = url_stream.try_next().await? {}

    Ok(())
}

static LOCKS: LazyLock<Arc<Mutex<Vec<String>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));

/// Return true if the url is locked and a watcher shouldn't be spawned for it
async fn is_url_locked(url: &InternetArchiveUrl) -> bool {
    LOCKS.lock().await.contains(&url.url)
}

/// Create a stream of jobs to watch
fn get_status_stream(
    conn: PgPool,
) -> impl Stream<Item = Result<InternetArchiveUrl, ArchivalError>> {
    try_fn_stream(async move |emitter| {
        let mut interval = time::interval(SETTINGS.status_watch_task.get_job_interval());

        loop {
            let jobs = InternetArchiveUrl::get_pending_jobs(&conn).await?;

            let mut has_emitted = false;
            for url in jobs {
                if !is_url_locked(&url).await {
                    LOCKS.lock().await.push(url.url.clone());

                    has_emitted = true;
                    emitter.emit(url).await;
                    interval.tick().await;
                }
            }

            // If there was no wait, we force one before the next fetch
            if !has_emitted {
                interval.tick().await;
            }
        }
    })
}

// Check the status of an url until it either finishes, or max retries are achieved
async fn watch_status(conn: &PgPool, mut url: InternetArchiveUrl) -> Result<(), ArchivalError> {
    let Some(job_id) = url.job_id.as_ref().cloned() else {
        return Ok(());
    };

    for _ in 0..SETTINGS.status_watch_task.max_retry {
        if check_status(conn, &mut url, &job_id).await? {
            LOCKS.lock().await.retain(|lock| lock != &url.url);

            return Ok(());
        }
    }

    // If we haven't succeeded or errored out by the last retry, we error out
    url.set_errored(conn).await?;

    warn!(
        "[Status watcher] Max retries for url status | id: {} | url: {}",
        url.id, url.url
    );

    LOCKS.lock().await.retain(|lock| lock != &url.url);

    Ok(())
}

/// Check the status of the archiving once
///
/// Return true if the job is done
async fn check_status(
    conn: &PgPool,
    url: &mut InternetArchiveUrl,
    job_id: &str,
) -> Result<bool, ArchivalError> {
    let status = archiving_job_status(job_id).await;

    match status {
        Ok(status) => handle_archival_response(conn, url, status).await,
        Err(ArchivalError::StatusRequestErrorResponse(err)) => {
            handle_archival_error(conn, url, err).await
        }
        Err(err) => Err(err),
    }
}

async fn handle_archival_response(
    conn: &PgPool,
    url: &mut InternetArchiveUrl,
    status: ArchivalStatusResponse,
) -> Result<bool, ArchivalError> {
    // Have we succeded?
    if status.status == "success" {
        url.set_archived(conn).await?;

        info!(
            "[Status watcher] Url archived successfully | id: {} | url: {}",
            url.id, url.url
        );

        Ok(true)
    }
    // Pending? Then we wait
    else if status.status == "pending" {
        Ok(false)
    } else {
        // We recieved something unexpected. Error out and try again
        error!(
            "[Status watcher] Recieved unknown status: {} | id: {} | url: {}",
            status.status, url.id, url.url
        );

        Ok(false)
    }
}

async fn handle_archival_error(
    conn: &PgPool,
    url: &mut InternetArchiveUrl,
    status: ArchivalStatusErrorResponse,
) -> Result<bool, ArchivalError> {
    // We got an error! Let's check whether it's recoverable
    if check_if_permanent_error(&status.status_ext) {
        url.set_failed(conn).await?;
    } else {
        url.set_errored(conn).await?;
    }

    warn!(
        "[Status watcher] Archiving error | id: {} | url: {} | error: {}",
        url.id, url.url, status.status_ext
    );

    Ok(true)
}
