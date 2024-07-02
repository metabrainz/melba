use colorize::AnsiColor;
use mb_rs::schema::{EditData, EditNote};
use sqlx::{Error, PgPool};
use crate::poller;

//TODO: Currently I am returning the internet_archive_urls row id when I insert any URL. Now there might be URLs which are already saved, hence instead of row id, show how many URLs are still there unprocessed, and is before the currently inserted one.
pub async fn insert_url_to_internet_archive_urls(
    url: String,
    pool: &PgPool,
) -> Result<i32, Error> {
    let id = sqlx::query!(
        r#"
        INSERT INTO external_url_archiver.internet_archive_urls (url, retry_count, is_saved)
        VALUES ($1, 0, false)
        RETURNING id
    "#,
        url
    ).fetch_one(pool)
        .await?
        .id;
    return Ok(id);
}

pub async fn insert_edit_data_row_to_internet_archive_urls(
    row_id: i32,
    pool: &PgPool
) -> Result<bool, Error> {
    let edit_data_row = sqlx::query_as::<_, EditData>(
        r#"
            SELECT * FROM edit_data
            WHERE edit = $1
        "#
    ).bind(row_id)
        .fetch_one(pool)
        .await.unwrap();

    let urls = poller::utils::extract_url_from_edit_data(&edit_data_row, pool).await;
    for url in &urls {
        let id = sqlx::query!(
        r#"
            INSERT INTO external_url_archiver.internet_archive_urls (url, from_table, from_table_id, retry_count, is_saved)
            VALUES ($1, 'edit_data', $2, 0, false)
            RETURNING id
        "#,
            url,
            edit_data_row.edit
        ).fetch_one(pool)
            .await?
            .id;
        println!("{} {} {}", "URL enqueued and id: ".green(), url, id);
    }
    Ok(!urls.is_empty())
}

pub async fn insert_edit_note_row_to_internet_archive_urls(
    row_id: i32,
    pool: &PgPool
) -> Result<bool, Error> {
    let edit_note_row = sqlx::query_as::<_, EditNote>(
        r#"
            SELECT * FROM edit_note
            WHERE id = $1
        "#
    ).bind(row_id)
        .fetch_one(pool)
        .await.unwrap();

    let urls = poller::utils::extract_urls_from_text(edit_note_row.text.as_str());
    for url in &urls {
        let id = sqlx::query!(
        r#"
            INSERT INTO external_url_archiver.internet_archive_urls (url, from_table, from_table_id, retry_count, is_saved)
            VALUES ($1, 'edit_note', $2, 0, false)
            RETURNING id
        "#,
            url,
            edit_note_row.edit
        ).fetch_one(pool)
            .await?
            .id;
        println!("{} {} {}", "URL enqueued and id: ".green(), url, id);
    }
    Ok(!urls.is_empty())

}

pub async fn get_job_id_status(
    job_id: String,
    _pool: &PgPool
) -> Result<&str, Error> {
    println!("job_id: {},", job_id);
    Ok("")
}