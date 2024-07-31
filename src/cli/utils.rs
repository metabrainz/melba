use crate::poller;
use colorize::AnsiColor;
use mb_rs::schema::{EditData, EditNote};
use sqlx::{Error, PgPool};

//TODO: Currently I am returning the internet_archive_urls row id when I insert any URL. Now there might be URLs which are already saved, hence instead of row id, show how many URLs are still there unprocessed, and is before the currently inserted one.
pub async fn insert_url_to_internet_archive_urls(url: &str, pool: &PgPool) -> Result<i32, Error> {
    sqlx::query!(
        r#"
        INSERT INTO external_url_archiver.internet_archive_urls (url, retry_count)
        VALUES ($1, 0)
        RETURNING id
    "#,
        url
    )
    .fetch_one(pool)
    .await
    .map(|result| result.id)
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
        let id = sqlx::query!(
        r#"
            INSERT INTO external_url_archiver.internet_archive_urls (url, from_table, from_table_id, retry_count)
            VALUES ($1, 'edit_data', $2, 0)
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
        let id = sqlx::query!(
        r#"
            INSERT INTO external_url_archiver.internet_archive_urls (url, from_table, from_table_id, retry_count)
            VALUES ($1, 'edit_note', $2, 0)
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

pub async fn get_job_id_status(job_id: String, _pool: &PgPool) -> Result<&str, Error> {
    // TODO: Concider using &str for job_id?
    println!("job_id: {},", job_id);
    Ok("")
}
