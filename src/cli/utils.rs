use crate::archival::archival_response::ArchivalStatusResponse;
use crate::archival::error::ArchivalError;
use crate::archival::utils::make_archival_status_request;
use crate::models::musicbrainz_db::EditData;
use crate::models::musicbrainz_db::EditNote;
use crate::poller;
use crate::poller::utils::should_insert_url_to_internet_archive_urls;
use colorize::AnsiColor;
use sqlx::{Error, PgPool, Row};

//TODO: Currently I am returning the internet_archive_urls row id when I insert any URL. Now there might be URLs which are already saved, hence instead of row id, show how many URLs are still there unprocessed, and is before the currently inserted one.
pub async fn insert_url_to_internet_archive_urls(
    url: &str,
    pool: &PgPool,
) -> Result<i32, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO external_url_archiver.internet_archive_urls (url, retry_count)
        VALUES ($1, 0)
        RETURNING id
        "#,
    )
    .bind(url)
    .fetch_one(pool)
    .await?;

    // Get the id from the returned row
    let id: i32 = row.try_get("id")?;

    Ok(id)
}

pub async fn check_before_inserting_url(url: &str, pool: &PgPool) -> Result<bool, Error> {
    should_insert_url_to_internet_archive_urls(url, pool).await
}

/// This function takes in an `edit_data` `row_id`, extract the urls contained inside, then insert them into the `internet_archive_urls` table
pub async fn insert_edit_data_row_to_internet_archive_urls(
    row_id: i32,
    pool: &PgPool,
) -> Result<bool, Error> {
    let edit_data_row = sqlx::query_as::<_, EditData>(
        r#"
            SELECT * FROM edit_data
            WHERE edit = $1
        "#,
    )
    .bind(row_id)
    .fetch_one(pool)
    .await?;

    let urls = poller::utils::extract_url_from_edit_data(&edit_data_row, pool).await;
    for url in &urls {
        let row = sqlx::query(
            r#"
            INSERT INTO external_url_archiver.internet_archive_urls (url, from_table, from_table_id, retry_count)
            VALUES ($1, 'edit_data', $2, 0)
            RETURNING id
        "#
        )
            .bind(url)
            .bind(edit_data_row.edit)
            .fetch_one(pool)
            .await?;
        let id: i32 = row.try_get("id")?;

        println!("{} {} {}", "URL enqueued and id: ".green(), url, id);
    }
    Ok(!urls.is_empty())
}

/// This function takes in an `edit_note` `row_id`, extract the urls contained inside, then insert them into the `internet_archive_urls` table
pub async fn insert_edit_note_row_to_internet_archive_urls(
    row_id: i32,
    pool: &PgPool,
) -> Result<bool, Error> {
    let edit_note_row = sqlx::query_as::<_, EditNote>(
        r#"
            SELECT * FROM edit_note
            WHERE id = $1
        "#,
    )
    .bind(row_id)
    .fetch_one(pool)
    .await?;

    let urls = poller::utils::extract_urls_from_text(&edit_note_row.text);
    for url in &urls {
        let row = sqlx::query(
            r#"
            INSERT INTO external_url_archiver.internet_archive_urls (url, from_table, from_table_id, retry_count)
            VALUES ($1, 'edit_note', $2, 0)
            RETURNING id
        "#
        )
            .bind(url)
            .bind(edit_note_row.edit)
            .fetch_one(pool)
            .await?;

        let id: i32 = row.try_get("id")?;
        println!("{} {} {}", "URL enqueued and id: ".green(), url, id);
    }
    Ok(!urls.is_empty())
}

pub async fn get_job_id_status(
    job_id: &str,
    _pool: &PgPool,
) -> Result<ArchivalStatusResponse, ArchivalError> {
    let status = make_archival_status_request(job_id).await?;
    Ok(status)
}
