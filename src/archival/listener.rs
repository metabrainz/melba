use crate::archival::utils::{
    make_archival_network_request, update_internet_archive_urls_with_job_id,
    update_internet_archive_urls_with_retry_count_inc,
};
use crate::structs::internet_archive_urls::InternetArchiveUrls;
use sqlx::postgres::PgListener;
use sqlx::PgPool;
use std::error::Error;

pub async fn listen(pool: PgPool) -> Result<(), Box<dyn Error>> {
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
) -> Result<(), Box<dyn Error>> {
    //TODO: make a reqwest here
    match make_archival_network_request(url.as_str()).await {
        Ok(job_id) => update_internet_archive_urls_with_job_id(pool, job_id, id).await?,
        Err(e) => {
            update_internet_archive_urls_with_retry_count_inc(pool, id).await?;
            println!("Error archiving url {} ,ERROR:  {}", url, e)
        }
    }
    Ok(())
}
