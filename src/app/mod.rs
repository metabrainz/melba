use crate::archival;
use crate::archival::notifier::Notifier;
use crate::poller::Poller;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::join;
use tokio::sync::Mutex;

/// Start the archiving service
pub async fn start(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Start all the tasks
    let (poll_result, notify_result, listener_result) = join!(
        spawn_poller_task(pool.clone()).await,
        spawn_notification_task(pool.clone()).await,
        spawn_archiver_task(pool.clone()).await
    );

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

/// Spawn the poller task. This task periodically check MusicBrainz's database for new URLs,
/// then add it to `internet_archive_urls` table
///
/// ⚠️ This must be awaited twice. Once to get the `JoinHandle`, and a second to start the task
async fn spawn_poller_task(db_pool: PgPool) -> tokio::task::JoinHandle<()> {
    const POLL_INTERVAL: u64 = 10;
    let mut poller = Poller::new(POLL_INTERVAL, db_pool.clone()).await;

    tokio::spawn(async move {
        poller.poll().await;
    })
}

/// Spawn the notification task. This task notify the archiver task that there is new URLs to be saved
///
/// ⚠️ This must be awaited twice. Once to get the `JoinHandle`, and a second to start the task
async fn spawn_notification_task(db_pool: PgPool) -> tokio::task::JoinHandle<()> {
    let notifier = Arc::new(Mutex::new(Notifier::new(db_pool.clone()).await));

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));

        while !db_pool.is_closed() {
            interval.tick().await;
            let notifier = Arc::clone(&notifier);
            let mut notifier = notifier.lock().await;

            if notifier.should_notify().await {
                println!("Notifying");

                if let Err(e) = notifier.notify().await {
                    //TODO: Handle Error and/or crash the app
                    eprintln!("Notify failed, error: {}", e)
                };
            }
        }
    })
}

/// Spawn the archiver task. It will listen for new URLs in the database, then request IA to archive them
///
/// ⚠️ This must be awaited twice. Once to get the `JoinHandle`, and a second to start the task
async fn spawn_archiver_task(db_pool: PgPool) -> tokio::task::JoinHandle<Result<(), sqlx::Error>> {
    tokio::spawn(async move { archival::listener::listen(db_pool).await })
}
