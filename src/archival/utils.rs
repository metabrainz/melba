use crate::archival::archival_response::{ArchivalHtmlResponse, ArchivalResponse, ArchivalStatusResponse};
use crate::archival::client::REQWEST_CLIENT;
use crate::archival::error::ArchivalError;
use crate::configuration::Settings;
use crate::structs::internet_archive_urls::InternetArchiveUrls;
use std::sync::Arc;
use reqwest::{header, Client};
use sqlx::{Error, PgPool};
use std::time::Duration;
use tokio::time;

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
pub async fn set_job_id_ia_url(pool: &PgPool, job_id: String, id: i32) -> Result<(), Error> {
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

pub async fn inc_archive_request_retry_count(pool: &PgPool, id: i32) -> Result<(), Error> {
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

pub async fn make_archival_status_request(
    job_id: &str,
    endpoint_url: &str,
) -> Result<ArchivalStatusResponse, ArchivalError> {
    let settings = Settings::new().expect("Config settings are not configured properly");
    let mut headers = header::HeaderMap::new();
    headers.insert("Accept", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        format!(
            "LOW {}:{}",
            settings.wayback_machine_api.myaccesskey, settings.wayback_machine_api.mysecret
        )
        .parse()
        .unwrap(),
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
        .body(format!("job_id={}", job_id))
        .send()
        .await?;
    // let response_status = response.status();
    let response_text = response.text().await?;

    if let Ok(res) = serde_json::from_str::<ArchivalStatusResponse>(&response_text) {
        return Ok(res);
    }
    Ok(ArchivalStatusResponse::Html(ArchivalHtmlResponse {
        html: response_text,
    }))
}

pub async fn schedule_status_check(
    job_id: String,
    endpoint_url: &str,
    id: i32,
    pool: PgPool,
) -> Result<(), ArchivalError> {
    let settings = Settings::new().expect("Config settings are not configured properly");

    for attempt in 1..=3 {
        time::sleep(Duration::from_secs(
            settings.listen_task.sleep_status_interval,
        ))
        .await;
        match make_archival_status_request(job_id.as_str(), endpoint_url).await? {
            ArchivalStatusResponse::Ok(status_response) => {
                if status_response.status == "success" {
                    set_status_in_database(&pool, id, status_response.status).await?;
                    return Ok(());
                } else if status_response.status == "pending" {
                    println!(
                        "Job {} is still pending. Time: {:?}",
                        job_id,
                        chrono::Utc::now()
                    );
                } else {
                    println!(
                        "Job {} returned status '{}', retry count : {}",
                        job_id, status_response.status, attempt
                    );
                }
            }
            ArchivalStatusResponse::Err(e) => {
                eprintln!("Error making status check request: {:?}", e)
            }
            ArchivalStatusResponse::Html(message) => {
                println!(
                    "Job {} cannot be checked for status. Response message: {}",
                    job_id, message.html
                )
            }
        }
    }

    // After 3 attempts, if still not success, update the status with the last response or Error
    match make_archival_status_request(job_id.as_str(), endpoint_url).await? {
        ArchivalStatusResponse::Ok(status_response) => {
            set_status_in_database(&pool, id, status_response.status).await?;
        }
        ArchivalStatusResponse::Err(e) => {
            set_status_in_database(&pool, id, format!("Error: Could not save: {:?}", e)).await?;
            eprintln!("Error making final status check request: {:?}", e)
        }
        ArchivalStatusResponse::Html(message) => {
            set_status_in_database(
                &pool,
                id,
                format!("Error: Could not save: {}", message.html),
            )
            .await?;
            eprintln!("Error making final status check request: {}", message.html)
        }
    }
    Ok(())
}

pub async fn set_status_in_database(pool: &PgPool, id: i32, status: String) -> Result<(), Error> {
    let query = r#"
        UPDATE external_url_archiver.internet_archive_urls
        SET
        status = $1
        WHERE id = $2
        "#;
    sqlx::query(query)
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
#[path = "./tests/utils.rs"]
mod tests;
