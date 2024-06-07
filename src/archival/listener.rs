use sqlx::{Error, PgPool};
use sqlx::postgres::PgListener;
use crate::archival::utils::update_internet_archive_urls;
use crate::structs::internet_archive_urls::InternetArchiveUrls;

pub async fn listen(pool: PgPool) -> Result<(), Error> {
    println!("Listener Task");
    let mut listener = PgListener::connect_with(&pool).await?;
    listener.listen("archive_urls").await.unwrap();
    loop {
        let notification = listener.recv().await.unwrap();
        println!("Notification Payload: {}", notification.payload());
        let payload: InternetArchiveUrls = serde_json::from_str(notification.payload()).unwrap();
        archive(payload.id, payload.url.unwrap(), payload.retry_count.unwrap(), &pool).await;
    }
}

pub async fn archive(
    id: i32,
    url: String,
    retry_count: i32,
    pool: &PgPool
) {
    //TODO: make a reqwest here
    //NOTE: assuming the URL got saved,
    // update the job_id, and is_saved variables of the row
    // in internet_archive_urls table
    // Keeping mock job id for now TODO: update it
    let job_id = "MOCK_JOB_ID".to_string();
    update_internet_archive_urls(pool, job_id, id).await;
}