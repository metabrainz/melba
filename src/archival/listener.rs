use crate::archival::utils::{
    make_archival_network_request, update_internet_archive_urls_with_job_id,
    update_internet_archive_urls_with_retry_count_inc,
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
            archive(
                payload.id,
                payload.url.unwrap(),
                payload.retry_count.unwrap(),
                &pool,
            )
            .await?;
        }
    }
}

pub async fn archive(
    id: i32,
    url: String,
    _retry_count: i32,
    pool: &PgPool,
) -> Result<(), ArchivalError> {
    match make_archival_network_request(url.as_str()).await? {
        ArchivalResponse::Ok(success) => {
            update_internet_archive_urls_with_job_id(pool, success.job_id, id).await?
        }
        ArchivalResponse::Err(e) => {
            update_internet_archive_urls_with_retry_count_inc(pool, id).await?;
            println!("Error archiving url {} ,ERROR:  {}", url, e.message)
        }
        ArchivalResponse::Html(response) => {
            update_internet_archive_urls_with_retry_count_inc(pool, id).await?;
            println!("Error archiving url {}, ERROR: {}", url, response.html)
        }
    }
    Ok(())
}
