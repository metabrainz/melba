use sqlx::PgPool;
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
        utils::init_notify_archive_urls_postgres_function(&pool).await;
        println!("Notifies starts from : {}", last_unarchived_row_from_internet_archive_urls_table);
        return Notifier {
            start_notifier_from: last_unarchived_row_from_internet_archive_urls_table,
            pool
        }
    }
    pub async fn notify(&mut self) {
        let pool = self.pool.clone();
        sqlx::query("SELECT notify_archive_urls($1)")
            .bind(self.start_notifier_from)
            .execute(&pool)
            .await
            .unwrap();
        println!("[start_id from notify], {}", self.start_notifier_from);
        self.start_notifier_from = self.start_notifier_from + 2;
    }
}

