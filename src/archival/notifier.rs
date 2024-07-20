use crate::archival::utils::{get_first_id_to_start_notifier_from, is_row_exists};
use sqlx::{Error, PgPool};

pub struct Notifier {
    start_notifier_from: Option<i32>,
    pool: PgPool,
}

impl Notifier {
    pub async fn new(pool: PgPool) -> Notifier {
        let start_notifier_from = get_first_id_to_start_notifier_from(pool.clone()).await;
        if start_notifier_from.is_some() {
            println!("Notifies starts from : {}", start_notifier_from.unwrap());
        }
        Notifier {
            start_notifier_from,
            pool,
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
            println!(
                "[start_id from notify], {}",
                self.start_notifier_from.unwrap()
            );

            //Case: If the notifier reached the end of the row, and couldn't find any unarchived row in Internet Archives URL table, we will not increment the self.start_notifier_from count
            if is_row_exists(&pool, self.start_notifier_from.unwrap()).await {
                self.start_notifier_from = Some(self.start_notifier_from.unwrap() + 1);
            }
            Ok(())
        } else {
            //Case: It could be that there is no URL in InternetArchiveURL table when we call `notify`, so we check for the id here, to start notifier from it in the next notify call
            println!("[NOTIFIER] No row detected, checking again");
            self.start_notifier_from = get_first_id_to_start_notifier_from(self.pool.clone()).await;
            Ok(())
        }
    }

    /// Checks if the row to begin notifying from is present in `internet_archive_urls`
    pub async fn should_notify(&mut self) -> bool {
        if self.start_notifier_from.is_some() {
            is_row_exists(&self.pool, self.start_notifier_from.unwrap()).await
        } else {
            true
        }
    }
}
