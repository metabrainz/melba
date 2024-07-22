use crate::poller::utils::{
    extract_url_from_edit_data, extract_url_from_edit_note, save_url_to_internet_archive_urls,
};
use mb_rs::schema::{EditData, EditNote};
use sqlx::{Error, PgPool};
use tokio::try_join;

/// Function which runs on each poll and thus is responsible for:
/// 1. Extracting the URL containing rows from different tables
/// 2. Transform the rows accordingly
/// 3. Check if we can insert the row in `internet_archive_urls` table, and insert it to the table
///
/// If the poll is successful, return the new ids of (`edit_data`,`edit_note`) to start the new poll with
pub async fn poll_db(
    pool: &PgPool,
    edit_data_start_idx: i32,
    edit_note_start_idx: i32,
) -> Result<(Option<i32>, Option<i32>), Error> {
    println!(
        "EditNote: {}, EditData: {}",
        edit_note_start_idx, edit_data_start_idx
    );

    // Return the next ids of the last edit and notes for the next poll
    try_join!(
        poll_and_save_edit_data(edit_data_start_idx, pool),
        poll_and_save_note_data(edit_note_start_idx, pool)
    )
}

/// This function polls the database for new edit data, then save it
///
/// It returns the id of the next edit to poll
pub async fn poll_and_save_edit_data(start_id: i32, pool: &PgPool) -> Result<Option<i32>, Error> {
    let edits = sqlx::query_as::<_, EditData>(
        r#"
            SELECT DISTINCT ON (edit)
            *
            FROM edit_data
            WHERE edit >= $1
            ORDER BY edit
            LIMIT 10;
        "#,
    )
    .bind(start_id)
    .fetch_all(pool)
    .await?;

    println!("Edits ->");
    for edit in &edits {
        let urls = extract_url_from_edit_data(edit, pool).await;

        for url in urls {
            save_url_to_internet_archive_urls(url.as_str(), "edit_data", edit.edit, pool)
                .await
                .unwrap_or_else(|e| eprintln!("Error saving URL from edit: {}: {}", edit.edit, e)); //TODO: Refactor. unwrap_or_else maps an error value from E to F. It shouldn't be used for display or map to ()
            println!("{}", url);
        }
    }

    Ok(edits.last().map(|edit| edit.edit + 1))
}

/// This function polls the database for new note data, then save it
///
/// It returns the id of the next note to poll
pub async fn poll_and_save_note_data(start_id: i32, pool: &PgPool) -> Result<Option<i32>, Error> {
    let notes = sqlx::query_as::<_, EditNote>(
        r#"
             SELECT DISTINCT ON (id)
            *
            FROM edit_note
            WHERE id >= $1
            ORDER BY id
            LIMIT 10;
        "#,
    )
    .bind(start_id)
    .fetch_all(pool)
    .await?;

    println!("Edit Notes ->");
    for note in &notes {
        let urls = extract_url_from_edit_note(note, pool).await;
        for url in urls {
            save_url_to_internet_archive_urls(url.as_str(), "edit_note", note.id, pool)
                .await
                .unwrap_or_else(|e| {
                    eprintln!("Error saving URL from edit note: {}: {}", note.id, e)
                    //TODO: Refactor. unwrap_or_else maps an error value from E to F. It shouldn't be used for display or map to ()
                });
            println!("{}", url);
        }
    }

    Ok(notes.last().map(|note| note.id + 1))
}
