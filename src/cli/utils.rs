use sqlx::{Error, PgPool};

pub async fn insert_url_to_internet_archive_urls(
    url: String,
    pool: &PgPool,
) {
    //todo
}

pub async fn insert_edit_data_row_to_internet_archive_urls(
    row_id: i32,
    pool: &PgPool
) {

}

pub async fn insert_edit_note_row_to_internet_archive_urls(
    row_id: i32,
    pool: &PgPool
) {

}

pub async fn get_job_id_status(
    job_id: String,
    pool: &PgPool
) -> Result<&str, Error> {
    Ok("")
}
