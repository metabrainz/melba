use crate::archival::archival_response;
use crate::archival::archival_response::{
    ArchivalResponse, ArchivalStatusErrorResponse, ArchivalStatusResponse,
};
use crate::archival::client::REQWEST_CLIENT;
use crate::archival::error::ArchivalError;
use crate::archival::error::ArchivalError::SaveRequestError;
use crate::configuration::SETTINGS;
use crate::metrics::Metrics;
use crate::structs::internet_archive_urls::{ArchivalStatus, InternetArchiveUrls};
use log::{debug, info, warn};
use sqlx::{Error, PgPool};
use std::time::Duration;
use tokio::time;

#[cfg(not(test))]
const SAVE_ENDPOINT_URL: &str = "http://web.archive.org/save";
#[cfg(not(test))]
const STATUS_ENDPOINT_URL: &str = "http://web.archive.org/save/status";

#[cfg(test)]
const SAVE_ENDPOINT_URL: &str = "http://127.0.0.1:1234/save";
#[cfg(test)]
const STATUS_ENDPOINT_URL: &str = "http://127.0.0.1:1235/status";

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
            debug!(
                "[NOTIFIER] No new row to notify in internet_archive_urls. Current id: {}. Reason: {:?}",
                row_id,
                error
            );
            false
        }
    }
}

///Handles the network request to archive the URL
pub async fn make_archival_network_request(url: &str) -> Result<ArchivalResponse, ArchivalError> {
    let client = &REQWEST_CLIENT;
    let response = client
        .post(SAVE_ENDPOINT_URL)
        .body(format!("url={}", url))
        .send()
        .await?;
    let response_text = response.text().await?;
    // Success response, contains job_id
    if let Ok(res) = serde_json::from_str::<ArchivalResponse>(&response_text) {
        return Ok(res);
    }
    if let Ok(e) = serde_json::from_str::<archival_response::ArchivalErrorResponse>(&response_text)
    {
        return Err(SaveRequestError(e));
    }
    // HTML response, case when IA can not archive URL, and is under maintenance
    Err(ArchivalError::HtmlResponse(response_text))
}

///Checks the status of `job_id` of a URL
pub async fn make_archival_status_request(
    job_id: &str,
) -> Result<ArchivalStatusResponse, ArchivalError> {
    let client = &REQWEST_CLIENT;
    let response = client
        .post(STATUS_ENDPOINT_URL)
        .body(format!("job_id={}", job_id))
        .send()
        .await?;
    // let response_status = response.status();
    let response_text = response.text().await?;

    if let Ok(res) = serde_json::from_str::<ArchivalStatusResponse>(&response_text) {
        if res.status != "error" {
            return Ok(res);
        }
    }
    if let Ok(e) = serde_json::from_str::<ArchivalStatusErrorResponse>(&response_text) {
        return Err(ArchivalError::StatusRequestErrorResponse(e));
    }
    Err(ArchivalError::HtmlResponse(response_text))
}

///Schedules the status check of a URL's `job_id`, and
pub async fn schedule_status_check(
    job_id: String,
    id: i32,
    pool: &PgPool,
) -> Result<(), ArchivalError> {
    let metrics = Metrics::new().await;
    metrics.network_request_counter.inc();
    metrics.push_metrics().await;
    info!(
        "[LISTENER] STATUS CHECK: Attempting status check for internet_archive_urls id: {} and job_id {}",
        id, job_id
    );
    set_status_with_message(pool, id, ArchivalStatus::Processing as i32, "Processing").await?;
    for attempt in 1..=3 {
        time::sleep(Duration::from_secs(
            SETTINGS.listen_task.sleep_status_interval,
        ))
        .await;
        let archival_status_response = make_archival_status_request(job_id.as_str()).await?;
        if archival_status_response.status == "success" {
            set_status_with_message(
                pool,
                id,
                ArchivalStatus::Success as i32,
                archival_status_response.status.as_str(),
            )
            .await?;
            metrics.record_archival_status("success archival").await;
            info!(
                "[LISTENER] STATUS CHECK: internet_archive_urls id: {} and job_id {} archived successfully",
                id, job_id
            );
            return Ok(());
        } else if attempt == 3 {
            let status = archival_status_response.status;
            warn!(
                "[LISTENER] STATUS CHECK: 3rd Attempt, no success for internet_archive_urls id: {} and job_id {}: status {:?}",
                id, job_id, &status
            );
            inc_archive_request_retry_count(pool, id).await?;
            set_status_with_message(
                pool,
                id,
                ArchivalStatus::StatusError as i32,
                status.as_str(),
            )
            .await?;
            metrics
                .record_archival_status("status attempt exceeded")
                .await;
        }
    }
    Ok(())
}

pub async fn set_status_with_message(
    pool: &PgPool,
    id: i32,
    status: i32,
    status_message: &str,
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

/// Job IDs that have permanent errors in `status_ext` will not be archived, hence we can remove them from `internet_archive_urls` table.
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
    permanent_errors
        .iter()
        .any(|&error| status_ext.contains(error))
}

#[cfg(test)]
#[path = "./tests/utils.rs"]
mod tests;
