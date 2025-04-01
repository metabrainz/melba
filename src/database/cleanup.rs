use sqlx::PgPool;
use tokio::time::interval;

use crate::configuration::SETTINGS;
use crate::models::melba::internet_archive_urls::InternetArchiveUrl;

/// Start the cleanup task.
///
/// This task cleanup the database from unwanted rows and old data, as well as check the integrity of the data
///
/// Running it or not shouldn't impact the program, and only make the database use more disk space.
/// It's important to not rely on it, and do proper checks
pub async fn cleanup_task(conn: &PgPool) -> Result<(), sqlx::Error> {
    let mut interval = interval(SETTINGS.cleaner_task.get_job_interval());

    loop {
        cleanup_database(conn).await?;
        interval.tick().await;
    }
}

async fn cleanup_database(conn: &PgPool) -> Result<(), sqlx::Error> {
    // Remove all the failed URLs. No need to keep them
    InternetArchiveUrl::delete_failed(conn).await?;

    // Expired urls
    //
    // We clean all the urls past the max timeout set in the config.
    // We only remove waiting and failed urls, as we don't want to stop the running ones.
    InternetArchiveUrl::delete_waiting_and_expired(conn).await?;
    InternetArchiveUrl::delete_errored_and_expired(conn).await?;

    Ok(())
}
