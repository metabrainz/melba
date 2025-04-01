use log::error;
use sqlx::PgPool;
use tokio::join;
use tokio::task::JoinHandle;

use crate::archival::archiver::run_archiver_task;
use crate::archival::status_watcher::run_status_watcher;
use crate::configuration::SETTINGS;
use crate::database::cleanup::cleanup_task;
use crate::poller::Poller;

/// Start the archiving service
pub async fn start(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Start all the tasks
    let (poll_result, archiver_result, status_watcher_result, cleanup_result) = join!(
        spawn_poller_task(pool.clone()).await,
        run_archiver_task(pool),
        run_status_watcher(pool),
        cleanup_task(pool),
    );

    if let Err(e) = poll_result {
        error!("Polling task failed: {:?}", e);
    }
    if let Err(e) = archiver_result {
        error!("Notification task failed: {:?}", e);
    }
    if let Err(e) = status_watcher_result {
        error!("Listener task failed: {:?}", e);
    }
    if let Err(e) = cleanup_result {
        error!("Cleaner task failed: {:?}", e);
    }

    Ok(())
}

/// Spawn the poller task. This task periodically check MusicBrainz's database for new URLs,
/// then add it to `internet_archive_urls` table
///
/// ⚠️ This must be awaited twice. Once to get the `JoinHandle`, and a second to start the task
pub async fn spawn_poller_task(db_pool: PgPool) -> JoinHandle<()> {
    let mut poller = Poller::new(SETTINGS.poller_task.poll_interval, db_pool.clone())
        .await
        .expect("[POLLER] Could not find rows in edit rows to start poller");

    tokio::spawn(async move {
        if let Err(e) = poller.run().await {
            error!("[POLLER] Task Failed, Error: {}", e);
            sentry::capture_error(&e);
        }
    })
}
