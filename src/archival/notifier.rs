use sqlx::{Error, PgPool};
use crate::archival::utils;


pub struct Notifier {
    start_notifier_from: i32,
    pool: PgPool
}

impl Notifier {
    pub async fn new(
        pool: PgPool
    ) -> Notifier {
        let last_unarchived_row_from_internet_archive_urls_table =
            utils::get_last_unarchived_row_from_internet_archive_urls_table(pool.clone()).await;
        println!("Notifies starts from : {}", last_unarchived_row_from_internet_archive_urls_table);
        return Notifier {
            start_notifier_from: last_unarchived_row_from_internet_archive_urls_table,
            pool
        }
    }
    pub async fn notify(&mut self) -> Result<(), Error> {
        let pool = self.pool.clone();
        let rows: (i32, ) = sqlx::query_as::<_,(i32,)>("SELECT external_url_archiver.notify_archive_urls($1)")
            .bind(self.start_notifier_from)
            .fetch_one(&pool)
            .await?;
        let processed_rows = rows.0;
        println!("[start_id from notify], {} and add {}", self.start_notifier_from, processed_rows);
        self.start_notifier_from = self.start_notifier_from + processed_rows;
        Ok(())
    }
}

