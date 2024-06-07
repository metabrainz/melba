use sqlx::PgPool;
use crate::structs::internet_archive_urls::InternetArchiveUrls;

///This function is used to find the row in internet_archive_urls from where we can start the archival task
/// The notify function will start picking URLs from the returned row id
pub async fn get_last_unarchived_row_from_internet_archive_urls_table(
    pool: PgPool
) -> i32 {
    let last_row = sqlx::query_as::<_,InternetArchiveUrls>(
        "SELECT DISTINCT ON (id) * from internet_archive_urls where is_saved = false order by id limit 1 "
    )
        .fetch_one(&pool)
        .await
        .unwrap();
    return last_row.id;
}
//TODO: make the function more flexible, add options to pass the limit as well
///This function creates a postgres function that starting from an index,
/// loops throw rows, and notifies a channel with the relevant data.
/// The postgres function is supposed to be called from the task which intends to archive the urls.
pub async fn init_notify_archive_urls_postgres_function(
    pool: &PgPool
) {
    let query = "
    CREATE OR REPLACE FUNCTION notify_archive_urls(start_id INTEGER) RETURNS VOID AS $$
    DECLARE
        rec RECORD;
    BEGIN
        FOR rec IN SELECT * FROM internet_archive_urls WHERE id >= start_id ORDER BY id LIMIT 2
        LOOP
            PERFORM pg_notify('archive_urls', row_to_json(rec)::text);
        END LOOP;
    END;
    $$ LANGUAGE plpgsql";
    sqlx::query(query)
        .execute(pool)
        .await
        .unwrap();
}

pub async fn update_internet_archive_urls(
    pool: &PgPool,
    job_id: String,
    id: i32
) {
    let query = r#"UPDATE internet_archive_urls SET
     is_saved = true,
     job_id = $1
     WHERE id = $2
     "#;
    sqlx::query(query)
        .bind(job_id)
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
}