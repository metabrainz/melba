use std::time::Duration;
use sqlx::postgres::PgPoolOptions;
use tokio::join;
use crate::poller::Poller;

mod poller;
mod archival;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const POLL_INTERVAL: u64 = 10;
    //NOTE: for time being, keeping the db_url to a custom db, will check with mb schema later
    let db_url = "postgres://musicbrainz:musicbrainz@localhost:5432/musicbrainz_db";
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();
    let notifier_pool = pool.clone();
    let listener_pool = pool.clone();
    let mut poller = Poller::new(POLL_INTERVAL, pool.clone());
    let poll_task_handler =
        tokio::spawn(async move {
        poller
            .await
            .poll()
            .await;
    });

    let notify_task_handler =
        tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        while !notifier_pool.is_closed() {
            interval.tick().await;
            archival::notifier::notify(&notifier_pool)
                .await;
        }
    });
    let listener_task_handler =
        tokio::spawn(async move {
        archival::listener::listen( listener_pool)
            .await
            .unwrap();
    });
    join!(poll_task_handler,notify_task_handler,listener_task_handler);
    Ok(())
}
