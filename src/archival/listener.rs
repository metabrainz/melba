use crate::archival::utils::{
    inc_archive_request_retry_count, make_archival_network_request, schedule_status_check,
    set_job_id_ia_url, set_status,
};

use crate::archival::archival_response::ArchivalResponse;
use crate::archival::error::ArchivalError;
use crate::configuration::Settings;
use crate::structs::internet_archive_urls::InternetArchiveUrls;
use sqlx::postgres::PgListener;
use sqlx::PgPool;
use std::time::Duration;
use tokio::time;

/// Listens to the `archive_urls` postgres channel
pub async fn listen(pool: PgPool) -> Result<(), ArchivalError> {
    println!("Listener Task");

    let settings = Settings::new().expect("Config settings not configured properly");

    let mut listener = PgListener::connect_with(&pool).await?;
    listener.listen("archive_urls").await?;
    loop {
        while let Some(notification) = listener.try_recv().await? {
            time::sleep(Duration::from_secs(settings.listen_task.listen_interval)).await;
            println!("Notification Payload: {}", notification.payload());
            let payload: InternetArchiveUrls =
                serde_json::from_str(notification.payload()).unwrap();
            handle_payload(payload, &pool).await?
        }
    }
}

/// Handle what to do with the URL, based on the retry count, either we try to archive, save as failed, or increment the retry count
pub async fn handle_payload(url: InternetArchiveUrls, pool: &PgPool) -> Result<(), ArchivalError> {
    let id = url.id;
    if url.retry_count >= Some(3) {
        set_status(pool, id, "Failed".to_string()).await?;
    } else if let Err(e) = archive(url, pool).await {
        eprintln!("Archival Error: {}", e);
        inc_archive_request_retry_count(pool, id).await?;
    }
    Ok(())
}

/// Send archival request, and schedule a status check request after `sleep_status_interval` seconds
pub async fn archive(
    internet_archive_urls_row: InternetArchiveUrls,
    pool: &PgPool,
) -> Result<(), ArchivalError> {
    let url = internet_archive_urls_row.url.unwrap();
    let id = internet_archive_urls_row.id;

    match make_archival_network_request(url.as_str(), "https://web.archive.org/save").await? {
        // If the response contains job id, we check for status
        ArchivalResponse::Ok(success) => {
            set_job_id_ia_url(pool, success.job_id.clone(), id).await?;
            let job_id = success.job_id.clone();
            let status_pool = pool.clone();
            tokio::spawn(async move {
                if let Err(e) = schedule_status_check(
                    job_id,
                    "https://web.archive.org/save/status",
                    id,
                    status_pool,
                )
                .await
                {
                    eprintln!("Error checking status: {}", e);
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
            )
        }
    }
    Ok(())
}
