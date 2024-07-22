use crate::archival::archival_response::{ArchivalHtmlResponse, ArchivalResponse};
use crate::archival::client::REQWEST_CLIENT;
use crate::archival::error::ArchivalError;
use crate::structs::internet_archive_urls::InternetArchiveUrls;
use sqlx::PgPool;
use std::sync::Arc;

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
        Some(last_row.id)
    } else {
        None
    }
}

/// Updates a row in `internet_archive_urls` table with the `job_id` response received from `Wayback Machine API` request, and marks `is_saved` true.
pub async fn set_job_id_ia_url(pool: &PgPool, job_id: String, id: i32) -> Result<(), sqlx::Error> {
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
        .await?;
    Ok(())
}

pub async fn inc_archive_request_retry_count(pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    let query = r#"
        UPDATE external_url_archiver.internet_archive_urls
        SET
        retry_count = retry_count + 1
        WHERE id = $1
     "#;
    sqlx::query(query).bind(id).execute(pool).await?;
    Ok(())
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
        Ok(_) => true,
        Err(error) => {
            println!("Cannot notify: {:?}", error);
            false
        }
    }
}

pub async fn make_archival_network_request(
    url: &str,
    endpoint_url: &str,
) -> Result<ArchivalResponse, ArchivalError> {
    let client = Arc::clone(&REQWEST_CLIENT);
    let response = client
        .post(endpoint_url)
        .body(format!("url={}", url))
        .send()
        .await?;
    let response_text = response.text().await?;
    // Success response, contains job_id
    if let Ok(res) = serde_json::from_str::<ArchivalResponse>(&response_text) {
        return Ok(res);
    }
    // HTML response, case when IA can not archive URL, and is under maintenance
    Ok(ArchivalResponse::Html(ArchivalHtmlResponse {
        html: response_text,
    }))
}

#[cfg(test)]
#[path = "./tests/utils.rs"]
mod tests;
