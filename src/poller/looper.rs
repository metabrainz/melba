use crate::debug_println;
use crate::metrics::Metrics;
use crate::poller::utils::{
    extract_url_from_edit_data, extract_url_from_edit_note, save_url_to_internet_archive_urls,
};
use mb_rs::schema::{EditData, EditNote};
use sqlx::{Error, PgPool};

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
    debug_println!(
        "[POLLER] Starting Polling from EditNote: {}, EditData: {}",
        edit_note_start_idx,
        edit_data_start_idx
    );
    let metrics = Metrics::new().await;
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
    .bind(edit_data_start_idx)
    .fetch_all(pool)
    .await?;

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
    .bind(edit_note_start_idx)
    .fetch_all(pool)
    .await?;

    for edit in &edits {
        let urls = extract_url_from_edit_data(edit, pool).await;
        for url in urls {
            let save_edit_data_url_result =
                save_url_to_internet_archive_urls(url.as_str(), "edit_data", edit.edit, pool).await;
            if let Ok(true) = save_edit_data_url_result {
                debug_println!("[POLLER] ADDED: Edit Data {} URL {}", edit.edit, url);
            } else if let Err(e) = save_edit_data_url_result {
                eprintln!("[POLLER] Error saving URL from edit: {}: {}", edit.edit, e)
            }
        }
    }
    for note in &notes {
        let urls = extract_url_from_edit_note(note, pool).await;
        for url in urls {
            let save_edit_note_url_result =
                save_url_to_internet_archive_urls(url.as_str(), "edit_note", note.id, pool).await;
            if let Ok(true) = save_edit_note_url_result {
                debug_println!("[POLLER] ADDED: Edit Note ID {} URL {}", note.id, url);
            } else if let Err(e) = save_edit_note_url_result {
                eprintln!(
                    "[POLLER] Error saving URL from edit note: {}: {}",
                    note.id, e
                )
            }
        }
    }
    metrics.db_poll_counter.inc();
    metrics.push_metrics().await;

    // Return the next ids of the last edit and notes for the next poll
    Ok((
        edits.last().map(|edit| edit.edit + 1),
        notes.last().map(|note| note.id + 1),
    ))
}
