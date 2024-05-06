use mb_rs::schema::{EditData, EditNote};
use sqlx::{Error};
use crate::poller::utils::{extract_url_from_edit_data, extract_urls_from_edit_note};

pub async fn poll_db(
    pool: &sqlx::PgPool
) -> Result<(), Error> {
    let edits = sqlx::query_as::<_, EditData>(
        "SELECT * FROM edit_data LIMIT 10")
        .fetch_all(pool)
        .await?;

    let notes = sqlx::query_as::<_, EditNote>(
        "SELECT * FROM edit_note LIMIT 10")
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