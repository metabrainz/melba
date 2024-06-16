use std::env;
use std::sync::{Arc};
use std::time::Duration;
use dotenv::dotenv;
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

    dotenv().ok();

    let hostname = env::var("PGHOST").expect("PGHOST environmental variable is not set");

    const POLL_INTERVAL: u64 = 10;
    //TODO: How to manage prod DB and dev DB?
    let db_url = format!("postgres://musicbrainz:musicbrainz@{}:5432/musicbrainz_db", hostname);
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
                notifier.lock().await.notify().await;
            };
        });

    let listener_task_handler =
        tokio::spawn(async move {
            archival::listener::listen(listener_pool)
                .await
                .unwrap();
        });
    let (poll_result, notify_result, listener_result) =
        join!(poll_task_handler,notify_task_handler,listener_task_handler);

    if let Err(e) = poll_result {
        eprintln!("Polling task failed: {:?}", e);
    }
    if let Err(e) = notify_result {
        eprintln!("Notification task failed: {:?}", e);
    }
    if let Err(e) = listener_result {
        eprintln!("Listener task failed: {:?}", e);
    }

    Ok(())
}
