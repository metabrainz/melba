use crate::archival::utils::{
    inc_archive_request_retry_count, make_archival_network_request, schedule_status_check,
    set_job_id_ia_url,
};
use crate::structs::archival_network_response::ArchivalResponse;
use crate::structs::error::ArchivalError;
use crate::structs::internet_archive_urls::InternetArchiveUrls;
use sqlx::postgres::PgListener;
use sqlx::PgPool;

pub async fn listen(pool: PgPool) -> Result<(), ArchivalError> {
    println!("Listener Task");
    let mut listener = PgListener::connect_with(&pool).await?;
    listener.listen("archive_urls").await.unwrap();
    loop {
        while let Some(notification) = listener.try_recv().await? {
            println!("Notification Payload: {}", notification.payload());
            let payload: InternetArchiveUrls =
                serde_json::from_str(notification.payload()).unwrap();
            if let Err(e) = archive(payload, &pool).await {
                eprintln!("Archival Error: {}", e)
            }
        }
    }
}

pub async fn archive(
    internet_archive_urls_row: InternetArchiveUrls,
    pool: &PgPool,
) -> Result<(), ArchivalError> {
    let url = internet_archive_urls_row.url.unwrap();
    let id = internet_archive_urls_row.id;
    match make_archival_network_request(url.as_str(), "https://web.archive.org/save").await? {
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
            println!("Error archiving url {}, ERROR: {}", url, response.html)
        }
    }
    Ok(())
}
