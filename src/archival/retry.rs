use crate::archival::utils::check_if_permanent_error;
use crate::configuration::SETTINGS;
use crate::structs::internet_archive_urls::{ArchivalStatus, InternetArchiveUrls};
use chrono::{Duration, Utc};
use log::info;
use sqlx::{Error, PgPool};
use std::ops::Sub;

/// Method started by `retry_and_cleanup` task, which reiterates `internet_archive_urls`, and according to the conditions, either re archives or cleans the row
pub async fn start(pool: PgPool) -> Result<(), Error> {
    let select_limit = SETTINGS.retry_task.select_limit;
    let mut last_id = 0;
    loop {
        let query = format!(
            r#"
        SELECT *
        FROM external_url_archiver.internet_archive_urls
        WHERE id > {}
        ORDER BY id ASC
        LIMIT {} "#,
            last_id, select_limit
        );
        let rows = sqlx::query_as::<_, InternetArchiveUrls>(&query)
            .fetch_all(&pool)
            .await?;
        if rows.is_empty() {
            break;
        }
        for row in rows {
            retry_and_cleanup_ia_row(row, &pool).await?;
        }
        last_id += select_limit;
    }
    info!("[RETRY_AND_CLEANUP] Task Complete");
    Ok(())
}

/// Given a row from `internet_archive_row, cleans it or retry archiving it
pub async fn retry_and_cleanup_ia_row(
    row: InternetArchiveUrls,
    pool: &PgPool,
) -> Result<(), Error> {
    let current_time = Utc::now();
    let created_at = row.created_at.unwrap();
    let duration_since_creation = current_time.sub(created_at);

    match row.status.try_into() {
        // If the URL status is failed, then we can remove it from the table (Case when still can't archive after 3 retries)
        Ok(ArchivalStatus::Failed) => {
            if duration_since_creation
                >= Duration::seconds(SETTINGS.retry_task.allow_remove_row_after)
            {
                sqlx::query(
                    "DELETE FROM external_url_archiver.internet_archive_urls WHERE id = $1",
                )
                .bind(row.id)
                .execute(pool)
                .await?;
                info!(
                    "[RETRY_AND_CLEANUP] Removing row {} due to failed URL: {} after {} seconds",
                    row.id,
                    row.url.unwrap_or("<null>".to_string()),
                    SETTINGS.retry_task.allow_remove_row_after
                )
            }
        }
        Ok(ArchivalStatus::StatusError) => {
            // If the URL cannot be archived due to Permanent errors, we can remove them from the table
            if let Some(status_ext) = row.status_message {
                if check_if_permanent_error(status_ext.as_str()) {
                    sqlx::query(
                        "DELETE FROM external_url_archiver.internet_archive_urls WHERE id = $1",
                    )
                    .bind(row.id)
                    .execute(pool)
                    .await?;
                    info!(
                        "[RETRY_AND_CLEANUP] Removing row {} containing URL: {} due to permanent error. status_ext: {}",
                        row.id,
                        row.url.unwrap_or("<null>".to_string()),
                        status_ext
                    )
                } else {
                    // If the archival status is null, which means the URL could not get archived earlier, therefore, enqueue the row to be sent to get archived
                    sqlx::query("SELECT external_url_archiver.notify_archive_urls($1)")
                        .bind(row.id)
                        .execute(pool)
                        .await?;
                    info!(
                        "[RETRY_AND_CLEANUP] Retrying notifying errored row {} containing URL: {}",
                        row.id,
                        row.url.unwrap_or("<null>".to_string())
                    )
                }
            }
        }
        _ => {
            // In any other case, if the URL has been there for more than the time limit, i.e. 24 hours, we will remove it, else re-archive it
            if duration_since_creation
                >= Duration::seconds(SETTINGS.retry_task.allow_remove_row_after)
            {
                sqlx::query(
                    "DELETE FROM external_url_archiver.internet_archive_urls WHERE id = $1",
                )
                .bind(row.id)
                .execute(pool)
                .await?;
                info!(
                    "[RETRY_AND_CLEANUP] Removing row {} containing URL: {} after {} seconds. Previous status: {} and status message: {}",
                    row.id,
                    row.url.unwrap_or("<null>".to_string()),
                    SETTINGS.retry_task.allow_remove_row_after,
                    row.status,
                    row.status_message.unwrap_or("<null>".to_string())
                )
            } else if row.status.try_into() != Ok(ArchivalStatus::Success) {
                sqlx::query("SELECT external_url_archiver.notify_archive_urls($1)")
                    .bind(row.id)
                    .execute(pool)
                    .await?;
                info!(
                    "[RETRY_AND_CLEANUP] Retrying row {} containing URL: {} after {} seconds. Previous status: {} and status message: {}",
                    row.id,
                    row.url.unwrap_or("<null>".to_string()),
                    SETTINGS.retry_task.allow_remove_row_after,
                    row.status,
                    row.status_message.unwrap_or("<null>".to_string())
                )
            }
        }
    }
    Ok(())
}
