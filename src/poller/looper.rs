use sqlx::{Error, Row};

pub async fn poll_db(
    pool: &sqlx::PgPool
) -> Result<(), Error> {
    //TODO: perform the edit_note and edit_data
    let rows = sqlx::query(
        "SELECT * FROM edit_data LIMIT 10")
        .fetch_all(pool)
        .await?;
    //TODO: transformations, and save transformed data to internet_archive_urls
    for row in rows {
        println!("{:?}", row.columns());
    }
    Ok(())
}