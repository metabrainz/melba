use std::ops::Deref;
use std::sync::{Arc};
use std::time::Duration;
use sqlx::postgres::PgPoolOptions;
use tokio::join;
use tokio::sync::Mutex;
use crate::archival::notifier::Notifier;
use crate::poller::Poller;

mod poller;
mod archival;
mod structs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const POLL_INTERVAL: u64 = 10;
    //TODO: How to manage prod DB and dev DB?
    let db_url = "postgres://musicbrainz:musicbrainz@localhost:5432/musicbrainz_db";
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    let notifier_pool = pool.clone();
    let listener_pool = pool.clone();

    let mut poller = Poller::new(POLL_INTERVAL, pool.clone()).await;
    let notifier = Arc::new(Mutex::new(Notifier::new(pool.clone()).await));
    let poll_task_handler =
        tokio::spawn(async move {
            poller
                .poll()
                .await;
        });

    let notify_task_handler =
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            while !notifier_pool.is_closed() {
                interval.tick().await;
                let notifier = Arc::clone(&notifier);
                let mut notifier_lock = notifier.lock();
                notifier_lock.await.notify().await;
            }
        });

    let listener_task_handler =
        tokio::spawn(async move {
            archival::listener::listen(listener_pool)
                .await
                .unwrap();
        });
    join!(poll_task_handler,notify_task_handler,listener_task_handler);
    Ok(())
}
