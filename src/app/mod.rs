use crate::archival;
use crate::archival::notifier::Notifier;
use crate::configuration::Settings;
use crate::poller::Poller;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::join;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// Start the archiving service
pub async fn start(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Start all the tasks
    let (poll_result, notify_result, listener_result, retry_and_cleanup_result) = join!(
        spawn_poller_task(pool.clone()).await,
        spawn_notification_task(pool.clone()).await,
        spawn_archiver_task(pool.clone()).await,
        spawn_retry_and_cleanup_task(pool.clone()).await
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
    if let Err(e) = retry_and_cleanup_result {
        eprintln!("Retry and cleanup failed: {:?}", e);
    }

    Ok(())
}

/// Spawn the poller task. This task periodically check MusicBrainz's database for new URLs,
/// then add it to `internet_archive_urls` table
///
/// ⚠️ This must be awaited twice. Once to get the `JoinHandle`, and a second to start the task
pub async fn spawn_poller_task(db_pool: PgPool) -> JoinHandle<()> {
    let settings = Settings::new().expect("Config settings are not configured properly");
    let mut poller = Poller::new(settings.poller_task.poll_interval, db_pool.clone())
        .await
        .expect("Could not find rows in edit rows to start poller");

    tokio::spawn(async move {
        if let Err(e) = poller.poll().await {
            sentry::capture_message(
                format!("Cannot poll edit data and edit notes, due to: {}", e).as_str(),
                sentry::Level::Warning,
            );
        }
    })
}

/// Spawn the notification task. This task notify the archiver task that there is new URLs to be saved
///
/// ⚠️ This must be awaited twice. Once to get the `JoinHandle`, and a second to start the task
async fn spawn_notification_task(db_pool: PgPool) -> JoinHandle<()> {
    let settings = Settings::new().expect("Config settings are not configured properly");
    let notifier = Arc::new(Mutex::new(Notifier::new(db_pool.clone()).await));

    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(Duration::from_secs(settings.notify_task.notify_interval));

        while !db_pool.is_closed() {
            interval.tick().await;
            let notifier = Arc::clone(&notifier);
            let mut notifier = notifier.lock().await;

            if notifier.should_notify().await {
                println!("Notifying");

                if let Err(e) = notifier.notify().await {
                    eprintln!("Notify failed, error: {}", e);
                    sentry::capture_error(&e);
                };
            }
        }
    })
}

/// Spawn the archiver task. It will listen for new URLs in the database, then request IA to archive them
///
/// ⚠️ This must be awaited twice. Once to get the `JoinHandle`, and a second to start the task
async fn spawn_archiver_task(db_pool: PgPool) -> JoinHandle<()> {
    tokio::spawn(async move {
        archival::listener::listen(db_pool)
            .await
            .unwrap_or_else(|e| {
                sentry::capture_error(&e);
                eprintln!("Listener Task Error {}", e)
            })
    })
}

/// Spawns the retry and cleanup task. It will iterate over `internet_archive_urls` table, and retry the unarchived URLs or clean the already archived ones.
///
/// ⚠️ This must be awaited twice. Once to get the `JoinHandle`, and a second to start the task
async fn spawn_retry_and_cleanup_task(db_pool: PgPool) -> JoinHandle<()> {
    let settings = Settings::new().expect("Config settings are not configured properly");
    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(Duration::from_secs(settings.retry_task.retry_interval));
        while !db_pool.is_closed() {
            interval.tick().await;
            sentry::capture_message("Retry and Cleanup Task started", sentry::Level::Info);
            let archival_retry_task = archival::retry::start(db_pool.clone()).await;
            match archival_retry_task {
                Ok(_) => {
                    sentry::capture_message(
                        "Retry and Cleanup Task Completed",
                        sentry::Level::Info,
                    );
                }
                Err(e) => {
                    sentry::capture_error(&e);
                    eprintln!("Retry and Cleanup Task failed, error: {}", e)
                }
            }
        }
    })
}
