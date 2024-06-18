use sqlx::PgPool;
use crate::structs::internet_archive_urls::InternetArchiveUrls;

///This function is used to find the row in internet_archive_urls from where we can start the archival task
/// The notify function will start picking URLs from the returned row id
pub async fn get_last_unarchived_row_from_internet_archive_urls_table(
    pool: PgPool
) -> i32 {
    let last_row = sqlx::query_as::<_,InternetArchiveUrls>(
        r#"SELECT DISTINCT ON (id) *
             FROM external_url_archiver.internet_archive_urls
             WHERE is_saved = false
             ORDER BY id
             LIMIT 1 "#
    )
        .fetch_one(&pool)
        .await
        .unwrap();
    return last_row.id;
}

pub async fn update_internet_archive_urls(
    pool: &PgPool,
    job_id: String,
    id: i32
) {
    let query = r#"UPDATE external_url_archiver.internet_archive_urls SET
     is_saved = true,
     job_id = $1
     WHERE id = $2
     "#;
    sqlx::query(query)
        .bind(job_id)
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
}