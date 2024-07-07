use crate::structs::internet_archive_urls::InternetArchiveUrls;
use sqlx::PgPool;

///This function is used to find the row in internet_archive_urls from where we can start the archival task
/// The notify function will start picking URLs from the returned row id
/// - returns `None` if no rows are present in the table
/// - else returns the `id` of the first unarchived row
pub async fn get_first_id_to_start_notifier_from(pool: PgPool) -> Option<i32> {
    let last_row_result = sqlx::query_as::<_, InternetArchiveUrls>(
        r#"
             SELECT DISTINCT ON (id) *
             FROM external_url_archiver.internet_archive_urls
             WHERE is_saved = false
             ORDER BY id
             LIMIT 1
             "#,
    )
    .fetch_one(&pool)
    .await;
    if let Ok(last_row) = last_row_result {
        return Some(last_row.id);
    } else {
        None
    }
}

/// Updates a row in `internet_archive_urls` table with the `job_id` response received from `Wayback Machine API` request, and marks `is_saved` true.
pub async fn update_internet_archive_urls(pool: &PgPool, job_id: String, id: i32) {
    let query = r#"
        UPDATE external_url_archiver.internet_archive_urls
        SET
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

pub async fn is_row_exists(pool: &PgPool, row_id: i32) -> bool {
    let query = r#"
        SELECT 1 FROM external_url_archiver.internet_archive_urls
        WHERE id = $1;
    "#;
    let is_row_exists_res = sqlx::query_as::<_, (i32,)>(query)
        .bind(row_id)
        .fetch_one(pool)
        .await;
    match is_row_exists_res {
        Ok(_) => return true,
        Err(error) => {
            println!("Cannot notify: {:?}", error);
            false
        }
    }
}

#[cfg(test)]
#[path = "./tests/utils.rs"]
mod tests;
