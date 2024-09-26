use crate::archival::utils::{
    inc_archive_request_retry_count, make_archival_network_request, schedule_status_check,
    set_job_id_ia_url, set_status_with_message,
};

use crate::archival::error::ArchivalError;
use crate::configuration::SETTINGS;
use crate::debug_println;
use crate::metrics::Metrics;
use crate::structs::internet_archive_urls::{ArchivalStatus, InternetArchiveUrls};
use sentry::Level::Error;
use sqlx::postgres::PgListener;
use sqlx::PgPool;
use std::time::Duration;
use tokio::time;

/// Listens to the `archive_urls` postgres channel
pub async fn listen(pool: PgPool) -> Result<(), ArchivalError> {
    let mut listener = PgListener::connect_with(&pool).await?;
    listener.listen("archive_urls").await?;
    loop {
        while let Some(notification) = listener.try_recv().await? {
            time::sleep(Duration::from_secs(SETTINGS.listen_task.listen_interval)).await;
            debug_println!(
                "[LISTENER] Notification Payload: {}",
                notification.payload()
            );
            let payload: InternetArchiveUrls = serde_json::from_str(notification.payload())?;
            handle_payload(payload, &pool).await?
        }
    }
}

/// Handle what to do with the URL when we listen it from postgres channel, based on the retry count, either we try to archive, save as failed, or increment the retry count
pub async fn handle_payload(
    url_row: InternetArchiveUrls,
    pool: &PgPool,
) -> Result<(), ArchivalError> {
    let metrics = Metrics::new().await;
    if let Some(url) = url_row.url {
        let id = url_row.id;
        if url_row.retry_count >= Some(3) {
            let status_message = url_row
                .status_message
                .unwrap_or("No status message present".to_string());
            let status_ext = format!(
                "FAILED Archival of URL {} , reason: {}",
                url, status_message
            );
            set_status_with_message(pool, id, ArchivalStatus::Failed as i32, status_ext.as_str())
                .await?;
            sentry::capture_message(status_ext.as_str(), Error);
            metrics.record_archival_status("failed").await;
        } else {
            let archival_result = archive(url, url_row.id, pool).await;
            if let Err(e) = archival_result {
                eprintln!("[LISTENER] Archival Error for id {}: {}", url_row.id, e);
                set_status_with_message(
                    pool,
                    id,
                    ArchivalStatus::StatusError as i32,
                    e.to_string().as_str(),
                )
                .await?;
                metrics.record_archival_status("error archival").await;
                inc_archive_request_retry_count(pool, id).await?;
            }
        }
    }
    Ok(())
}

/// Send archival request, and schedule a status check request after `sleep_status_interval` seconds
pub async fn archive(url: String, id: i32, pool: &PgPool) -> Result<(), ArchivalError> {
    let success = make_archival_network_request(url.as_str()).await?;
    set_job_id_ia_url(pool, success.job_id.clone(), id).await?;
    debug_println!("[LISTENER] ARCHIVAL REQUEST SUCCESSFUL: url: {},  internet_archive_url id: {} and Job Id: {}", url, id, success.job_id);
    let metrics = Metrics::new().await;
    metrics.record_archival_status("archival started").await;

    let job_id = success.job_id.clone();
    let status_pool = pool.clone();
    tokio::spawn(async move {
        let schedule_status_check_result = schedule_status_check(job_id, id, &status_pool).await;
        if let Err(e) = schedule_status_check_result {
            let metrics = Metrics::new().await;
            metrics
                .record_archival_status("error archival status")
                .await;
            inc_archive_request_retry_count(&status_pool, id)
                .await
                .unwrap_or_else(|e| {
                    eprintln!("[LISTENER] Could not increment archive request retry count for internet_archive_urls id: {}, error: {}", id, e);
                    sentry::capture_error(&e);
                });
            set_status_with_message(
                &status_pool,
                id,
                ArchivalStatus::StatusError as i32,
                e.to_string().as_str(),
            )
            .await
            .unwrap_or_else(|e| {
                eprintln!(
                    "[LISTENER] Could not set status for internet_archive_urls id: {}, error: {}",
                    id, e
                );
                sentry::capture_error(&e);
            });
            sentry::capture_error(&e);
        }
    });
    Ok(())
}
