use crate::metrics::Metrics;
use crate::models::musicbrainz_db::EditData;
use crate::models::musicbrainz_db::EditNote;
use crate::poller::utils::{
    extract_url_from_edit_data, extract_url_from_edit_note, save_url_to_internet_archive_urls,
};
use log::{info, warn};
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
    info!(
        "[POLLER] Starting Polling from EditNote: {}, EditData: {}",
        edit_note_start_idx, edit_data_start_idx
    );
    let metrics = Metrics::new().await;

    // Handle Edits
    let next_edit_id = poll_and_save_edit_data(edit_data_start_idx, pool).await?;

    // Handle Edits Notes
    let next_edit_note_id = poll_and_save_edit_note_data(edit_note_start_idx, pool).await?;

    // Perf Note: use join! on the two calls

    metrics.db_poll_counter.inc();
    metrics.push_metrics().await;

    // Return the next ids of the last edit and notes for the next poll
    Ok((next_edit_id, next_edit_note_id))
}

/// Poll the edit data from the database and save it
async fn poll_and_save_edit_data(start_id: i32, pool: &PgPool) -> Result<Option<i32>, Error> {
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

    // Perf Note: use a stream
    for edit in &edits {
        let urls = extract_url_from_edit_data(edit, pool).await;

        // Perf Note: use a stream
        for url in urls {
            let save_edit_data_url_result =
                save_url_to_internet_archive_urls(url.as_str(), "edit_data", edit.edit, pool).await;

            match save_edit_data_url_result {
                // The url got saved
                Ok(true) => info!("[POLLER] ADDED: Edit ID `{}` URL `{}`", edit.edit, url),

                // The url didn't need to get saved
                Ok(false) => {}

                // Couldn't save the URL
                Err(e) => warn!(
                    "[POLLER] Error saving an URL from edit id `{}`: {}",
                    edit.edit, e
                ),
            }
        }
    }

    Ok(edits.last().map(|edit| edit.edit + 1))
}

/// Poll the edit data from the database and save it
async fn poll_and_save_edit_note_data(start_id: i32, pool: &PgPool) -> Result<Option<i32>, Error> {
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

    // Perf Note: use a stream
    for note in &notes {
        let urls = extract_url_from_edit_note(note, pool).await;

        // Perf Note: use a stream
        for url in urls {
            let save_edit_note_url_result =
                save_url_to_internet_archive_urls(url.as_str(), "edit_note", note.id, pool).await;

            match save_edit_note_url_result {
                // The url got saved
                Ok(true) => info!("[POLLER] ADDED: Edit Note ID `{}` URL `{}`", note.id, url),

                // The url didn't need to get saved
                Ok(false) => {}

                // Couldn't save the URL
                Err(e) => warn!(
                    "[POLLER] Error saving an URL from edit note id `{}`: {}",
                    note.id, e
                ),
            }
        }
    }

    Ok(notes.last().map(|note| note.id + 1))
}
