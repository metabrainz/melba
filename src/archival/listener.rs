use crate::archival::utils::{
    inc_archive_request_retry_count, make_archival_network_request, set_job_id_ia_url,
};

use crate::archival::archival_response::ArchivalResponse;
use crate::archival::error::ArchivalError;
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
            if let Err(e) = archive(
                payload.id,
                payload.url.unwrap(),
                payload.retry_count.unwrap(),
                &pool,
            )
            .await
            {
                eprintln!("Archival Error: {}", e)
            }
        }
    }
}

pub async fn archive(
    id: i32,
    url: String,
    _retry_count: i32,
    pool: &PgPool,
) -> Result<(), ArchivalError> {
    match make_archival_network_request(url.as_str(), "https://web.archive.org/save").await? {
        ArchivalResponse::Ok(success) => set_job_id_ia_url(pool, success.job_id, id).await?,
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
