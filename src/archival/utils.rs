use crate::archival::archival_response::{
    ArchivalHtmlResponse, ArchivalResponse, ArchivalStatusResponse,
};
use crate::archival::client::REQWEST_CLIENT;
use crate::archival::error::ArchivalError;
use crate::configuration::Settings;
use crate::structs::internet_archive_urls::{ArchivalStatus, InternetArchiveUrls};
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
             WHERE status = 1
             ORDER BY id
             LIMIT 1
             "#,
    )
    .fetch_one(&pool)
    .await;
    println!("What {:?}", last_row_result);
    last_row_result.map(|last_row| last_row.id).ok()
}

/// Updates a row in `internet_archive_urls` table with the `job_id` response received from `Wayback Machine API` request, and marks `status` processing.
pub async fn set_job_id_ia_url(pool: &PgPool, job_id: String, id: i32) -> Result<(), Error> {
    let query = r#"
        UPDATE external_url_archiver.internet_archive_urls
        SET
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

///Handles the network request to archive the URL
pub async fn make_archival_network_request(
    url: &str,
    endpoint_url: &str,
) -> Result<ArchivalResponse, ArchivalError> {
    let client = &REQWEST_CLIENT;
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

///Checks the status of `job_id` of a URL
pub async fn make_archival_status_request(
    job_id: &str,
    endpoint_url: &str,
) -> Result<ArchivalStatusResponse, ArchivalError> {
    let client = &REQWEST_CLIENT;
    let response = client
        .post(endpoint_url)
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

///Schedules the status check of a URL's `job_id`, and
pub async fn schedule_status_check(
    job_id: String,
    endpoint_url: &str,
    id: i32,
    pool: &PgPool,
) -> Result<(), ArchivalError> {
    println!("What");
    let settings = Settings::new().expect("Config settings are not configured properly");
    println!("{}", job_id);
    set_status_with_message(
        pool,
        id,
        ArchivalStatus::Processing as i32,
        "Processing".to_string(),
    )
    .await
    .unwrap();
    for attempt in 1..=3 {
        time::sleep(Duration::from_secs(
            settings.listen_task.sleep_status_interval,
        ))
        .await;
        match make_archival_status_request(job_id.as_str(), endpoint_url).await? {
            ArchivalStatusResponse::Ok(status_response) => {
                if status_response.status == "success" {
                    set_status_with_message(
                        pool,
                        id,
                        ArchivalStatus::Success as i32,
                        status_response.status,
                    )
                    .await?;
                    return Ok(());
                } else {
                    if attempt == 3 {
                        let status = status_response.status;
                        eprintln!("Error making final status check request: {:?}", &status);
                        inc_archive_request_retry_count(pool, id).await.unwrap();
                        set_status_with_message(
                            pool,
                            id,
                            ArchivalStatus::StatusError as i32,
                            status,
                        )
                        .await?;
                    }
                    eprintln!("Could not archive: {} attempt", attempt)
                }
            }
            ArchivalStatusResponse::Err(e) => {
                eprintln!("Error making status check request: {:?}", e);
                if attempt == 3 {
                    //Set status error
                    inc_archive_request_retry_count(pool, id).await.unwrap();
                    set_status_with_message(
                        pool,
                        id,
                        ArchivalStatus::StatusError as i32,
                        e.status_ext,
                    )
                    .await?;
                    eprintln!(
                        "Error making final status check request: {:?}",
                        e.message.as_str()
                    )
                }
            }
            ArchivalStatusResponse::Html(message) => {
                eprintln!("Error making final status check request: {}", message.html);
                if attempt == 3 {
                    //Set status error
                    inc_archive_request_retry_count(pool, id).await.unwrap();
                    set_status_with_message(
                        pool,
                        id,
                        ArchivalStatus::StatusError as i32,
                        format!("Error: Could not save: {}", message.html),
                    )
                    .await?;
                    sentry::capture_message(
                        format!("Internet Archive is Not Working, {}", message.html).as_str(),
                        sentry::Level::Warning,
                    );
                }
            }
        }
    }
    Ok(())
}

pub async fn set_status_with_message(
    pool: &PgPool,
    id: i32,
    status: i32,
    status_message: String,
) -> Result<(), Error> {
    let query = r#"
        UPDATE external_url_archiver.internet_archive_urls
        SET
        status = $1,
        status_message = $2
        WHERE id = $3
        "#;
    sqlx::query(query)
        .bind(status)
        .bind(status_message)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn set_status(pool: &PgPool, id: i32, status: i32) -> Result<(), Error> {
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

pub fn check_if_permanent_error(status_ext: &str) -> bool {
    let permanent_errors = vec![
        "error:bad-request",
        "error:blocked-url",
        "error:blocked",
        "error:blocked-client-ip",
        "error:filesize-limit",
        "error:http-version-not-supported",
        "error:invalid-url-syntax",
        "error:invalid-host-resolution",
        "error:method-not-allowed",
        "error:not-implemented",
        "error:not-found",
        "error:no-access",
        "error:unauthorized",
    ];
    permanent_errors.iter().any(|&error| error == status_ext)
}

#[cfg(test)]
#[path = "./tests/utils.rs"]
mod tests;
