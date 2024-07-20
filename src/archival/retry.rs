use crate::configuration::Settings;
use crate::structs::internet_archive_urls::InternetArchiveUrls;
use chrono::{Duration, Utc};
use sqlx::{Error, PgPool};
use std::ops::Sub;

/// Method started by `retry_and_cleanup` task, which reiterates `internet_archive_urls`, and according to the conditions, either re archives or cleans the row
pub async fn start(pool: PgPool) -> Result<(), Error> {
    let settings = Settings::new().expect("Config settings not configured properly");
    let select_limit = settings.retry_task.select_limit;
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
        println!("Retry task: {}, {}", last_id, select_limit)
    }
    println!("Retry/Cleanup Task Complete");
    Ok(())
}

/// Given a row from `internet_archive_row, cleans it or retry archiving it
pub async fn retry_and_cleanup_ia_row(
    row: InternetArchiveUrls,
    pool: &PgPool,
) -> Result<(), Error> {
    let settings = Settings::new().expect("Config settings are not configured properly");
    if let Some(archival_status) = row.status {
        let current_time = Utc::now();
        let created_at = row.created_at.unwrap();
        let duration_since_creation = current_time.sub(created_at);

        // Delete the row if the archival status is success and the duration is 24 hours or more
        if archival_status == "success"
            && duration_since_creation
                >= Duration::seconds(settings.retry_task.allow_remove_row_after)
        {
            sqlx::query("DELETE FROM external_url_archiver.internet_archive_urls WHERE id = $1")
                .bind(row.id)
                .execute(pool)
                .await?;
        }
    } else {
        // If the archival status is null, which means the URL could not get archived earlier, therefore, enqueue the row to be sent to get archived
        sqlx::query("SELECT external_url_archiver.notify_archive_urls($1)")
            .bind(row.id)
            .execute(pool)
            .await?;
    }
    Ok(())
}
