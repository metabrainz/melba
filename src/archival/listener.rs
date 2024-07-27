use crate::archival::utils::{
    inc_archive_request_retry_count, make_archival_network_request, schedule_status_check,
    set_job_id_ia_url, set_status,
};

use crate::archival::archival_response::ArchivalResponse;
use crate::archival::error::ArchivalError;
use crate::configuration::Settings;
use crate::structs::internet_archive_urls::{ArchivalStatus, InternetArchiveUrls};
use sentry::Level::Error;
use sqlx::postgres::PgListener;
use sqlx::PgPool;
use std::time::Duration;
use tokio::time;

/// Listens to the `archive_urls` postgres channel
pub async fn listen(pool: PgPool) -> Result<(), ArchivalError> {
    let settings = Settings::new().expect("Config settings not configured properly");

    let mut listener = PgListener::connect_with(&pool).await?;
    listener.listen("archive_urls").await?;
    loop {
        while let Some(notification) = listener.try_recv().await? {
            time::sleep(Duration::from_secs(settings.listen_task.listen_interval)).await;
            println!("Notification Payload: {}", notification.payload());
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
            set_status(pool, id, ArchivalStatus::Failed as i32).await?;
            sentry::capture_message(status_ext.as_str(), Error);
        } else {
            let archival_result = archive(url, url_row.id, pool).await;
            if let Err(e) = archival_result {
                eprintln!("Archival Error : {}", e);
                inc_archive_request_retry_count(pool, id).await?;
            }
        }
    }
    Ok(())
}

/// Send archival request, and schedule a status check request after `sleep_status_interval` seconds
pub async fn archive(url: String, id: i32, pool: &PgPool) -> Result<(), ArchivalError> {
    match make_archival_network_request(url.as_str()).await? {
        // If the response contains job id, we check for status
        ArchivalResponse::Ok(success) => {
            set_job_id_ia_url(pool, success.job_id.clone(), id).await?;
            let job_id = success.job_id.clone();
            let status_pool = pool.clone();
            tokio::spawn(async move {
                let schedule_status_check_result =
                    schedule_status_check(job_id, id, &status_pool).await;
                if let Err(e) = schedule_status_check_result {
                    inc_archive_request_retry_count(&status_pool, id)
                        .await
                        .unwrap();
                    sentry::capture_error(&e);
                }
            });
        }
        ArchivalResponse::Err(e) => {
            inc_archive_request_retry_count(pool, id).await?;
            println!("Error archiving url {} ,ERROR:  {}", url, e.message)
        }
        ArchivalResponse::Html(response) => {
            inc_archive_request_retry_count(pool, id).await?;
            println!(
                "Internet Archive cannot archive currently {}, due to: {}",
                url, response.html
            );
            sentry::capture_message(
                format!("Internet Archive is Not Working, {}", response.html).as_str(),
                sentry::Level::Warning,
            );
        }
    }
    Ok(())
}
