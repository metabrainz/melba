use mb_rs::schema::{EditData, EditNote};
use sqlx::{Error};
use crate::poller::utils::{extract_url_from_edit_data, extract_urls_from_edit_note};

pub async fn poll_db(
    pool: &sqlx::PgPool,
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
        let extracted_data = extract_url_from_edit_data(edit.data);
        if extracted_data.is_some() {
            println!("{}", extracted_data.unwrap());
        }
    }
    println!("Edit Notes ->");
    for note in notes {
        let urls = extract_urls_from_edit_note(note.text.as_str());
        println!("{:?}", urls);
    }
    Ok(())
}