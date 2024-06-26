use sqlx::{Error, PgPool};
use crate::archival::utils::get_first_id_to_start_notifier_from;


pub struct Notifier {
    start_notifier_from: Option<i32>,
    pool: PgPool
}

impl Notifier {
    pub async fn new(
        pool: PgPool
    ) -> Notifier {
        let last_unarchived_row_from_internet_archive_urls_table =
            get_first_id_to_start_notifier_from(pool.clone()).await;
        if last_unarchived_row_from_internet_archive_urls_table.is_some() {
            println!("Notifies starts from : {}", last_unarchived_row_from_internet_archive_urls_table.unwrap());
        }
        return Notifier {
            start_notifier_from: last_unarchived_row_from_internet_archive_urls_table,
            pool
        }
    }

    ///`notify` function is called everytime we want to send the URLs from `internet_archive_urls` table to the `listener` task,
    /// which archives the URL through network request using WayBack Machine API
    pub async fn notify(&mut self) -> Result<(), Error> {
        if self.start_notifier_from.is_some() {
            let pool = self.pool.clone();
            sqlx::query("SELECT external_url_archiver.notify_archive_urls($1)")
                .bind(self.start_notifier_from)
                .execute(&pool)
                .await?;

            println!("[start_id from notify], {}", self.start_notifier_from.unwrap());

            //Case: If the notifier reached the end of the row, and couldn't find any unarchived row in Internet Archives URL table, we will not increment the self.start_notifier_from count
            if let Some(restart_notifier_from) = get_first_id_to_start_notifier_from(self.pool.clone()).await {
                self.start_notifier_from = Some(restart_notifier_from + 1)
            }
            Ok(())
        } else {
            //Case: It could be that there is no URL in InternetArchiveURL table when we call `notify`, so we check for the id here, to start notifier from it in the next notify call
            self.start_notifier_from = get_first_id_to_start_notifier_from(self.pool.clone()).await;
            Ok(())
        }
    }
}

