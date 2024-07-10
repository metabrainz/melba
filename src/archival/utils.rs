use crate::structs::archival_network_response::{ArchivalErrorResponse, ArchivalSuccessResponse};
use crate::structs::internet_archive_urls::InternetArchiveUrls;
use reqwest::{header, Client};
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
        Some(last_row.id)
    } else {
        None
    }
}

/// Updates a row in `internet_archive_urls` table with the `job_id` response received from `Wayback Machine API` request, and marks `is_saved` true.
pub async fn update_internet_archive_urls_with_job_id(
    pool: &PgPool,
    job_id: String,
    id: i32,
) -> Result<(), sqlx::Error> {
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

pub async fn update_internet_archive_urls_with_retry_count_inc(
    pool: &PgPool,
    id: i32,
) -> Result<(), sqlx::Error> {
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
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let endpoint_url = "https://web.archive.org/save";
    let mut headers = header::HeaderMap::new();
    headers.insert("Accept", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        "LOW iJN8ly6eMroQjKfd:TxLzPGdXKMWvLLuY".parse().unwrap(),
    );
    headers.insert(
        "Content-Type",
        "application/x-www-form-urlencoded".parse().unwrap(),
    );

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    let response = client
        .post(endpoint_url)
        .headers(headers)
        .body(format!("url={}", url))
        .send()
        .await?;
    let response_status = response.status();
    let response_text = response.text().await?;

    if let Ok(result_ok) = serde_json::from_str::<ArchivalSuccessResponse>(&response_text) {
        Ok(result_ok.job_id)
    } else if let Ok(result_error) = serde_json::from_str::<ArchivalErrorResponse>(&response_text) {
        Err(Box::from(format!(
            "WayBack Machine API Error: Status: {}, Message: {}",
            result_error.status, result_error.message
        )))
    } else {
        Err(Box::from(format!(
            "Response Error: Status - {}, Message Body: {}",
            response_status.as_str(),
            response_text
        )))
    }
}

#[cfg(test)]
#[path = "./tests/utils.rs"]
mod tests;
