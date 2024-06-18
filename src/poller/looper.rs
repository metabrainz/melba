use mb_rs::schema::{EditData, EditNote};
use sqlx::{Error, PgPool};
use crate::poller::utils::{extract_url_from_edit_data, extract_urls_from_text, should_insert_url_to_internet_archive_urls};

pub async fn poll_db(
    pool: &PgPool,
    edit_data_start_idx: i32,
    edit_note_start_idx: i32
) -> Result<(), Error> {
    println!("EditNote: {}, EditData: {}", edit_note_start_idx, edit_data_start_idx);
    let edits = sqlx::query_as::<_, EditData>(
        r#"SELECT * FROM edit_data WHERE "edit" > $1 ORDER BY edit LIMIT 10"#)
        .bind(edit_data_start_idx)
        .fetch_all(pool)
        .await?;
    let notes = sqlx::query_as::<_, EditNote>(
        r#"SELECT * FROM edit_note WHERE "id" > $1 ORDER BY id LIMIT 10"#)
        .bind(edit_note_start_idx)
        .fetch_all(pool)
        .await?;
    //TODO: transformations, and save transformed data to internet_archive_urls
    println!("Edits ->");
    for edit in edits {
        let urls = extract_url_from_edit_data(edit.data);
        for url in urls {
            save_url_to_internet_archive_urls(
                url.as_str(),
                "edit_data",
                edit.edit,
                pool
            ).await;
            println!("{}", url);
        }
    }
    println!("Edit Notes ->");
    for note in notes {
        let urls = extract_urls_from_text(note.text.as_str());
        for url in urls {
            save_url_to_internet_archive_urls(
                url.as_str(),
                "edit_note",
                note.id,
                pool
            ).await;
            println!("{}", url);
        }
    }
    Ok(())
}

pub async fn save_url_to_internet_archive_urls(
    url: &str,
    from_table: &str,
    from_table_id: i32,
    pool: &PgPool) {
    if should_insert_url_to_internet_archive_urls(url, pool).await.expect("Error: ") {
        let query = "INSERT INTO external_url_archiver.internet_archive_urls(url, from_table, from_table_id, retry_count, is_saved) VALUES ($1, $2, $3, 0, false)";
        sqlx::query(query)
            .bind(url)
            .bind(from_table)
            .bind(from_table_id)
            .execute(pool)
            .await
            .unwrap();
    } else {
        return;
    }
}
