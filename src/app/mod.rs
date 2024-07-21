use crate::archival;
use crate::archival::notifier::Notifier;
use crate::configuration::Settings;
use crate::poller::Poller;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::join;
use tokio::sync::Mutex;

pub async fn start(pool: &PgPool) -> Result<(), sqlx::Error> {
    let settings = Settings::new().expect("Config settings are not configured properly");

    let notifier_pool = pool.clone();
    let listener_pool = pool.clone();
    let retry_task_pool = pool.clone();

    let mut poller = Poller::new(settings.poller_task.poll_interval, pool.clone()).await;
    let notifier = Arc::new(Mutex::new(Notifier::new(pool.clone()).await));

    let poll_task_handler = tokio::spawn(async move {
        poller.poll().await;
    });

    let notify_task_handler = tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(Duration::from_secs(settings.notify_task.notify_interval));
        while !notifier_pool.is_closed() {
            interval.tick().await;
            let notifier = Arc::clone(&notifier);
            let mut notifier = notifier.lock().await;
            if notifier.should_notify().await {
                println!("Notifying...");
                if let Err(e) = notifier.notify().await {
                    eprintln!("Notify failed, error: {}", e)
                };
            }
        }
    });

    let listener_task_handler = tokio::spawn(async move {
        archival::listener::listen(listener_pool)
            .await
            .map_err(|e| eprintln!("Listener Task Error {}", e))
    });

    let retry_and_cleanup_task_handler = tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(Duration::from_secs(settings.retry_task.retry_interval));
        while !retry_task_pool.is_closed() {
            interval.tick().await;
            println!("Retrying and Cleanup Task Starting...");
            if let Err(e) = archival::retry::start(retry_task_pool.clone()).await {
                eprintln!("Retry and Cleanup Task failed, error: {}", e)
            }
        }
    });

    let (poll_result, notify_result, listener_result, retry_and_cleanup_result) = join!(
        poll_task_handler,
        notify_task_handler,
        listener_task_handler,
        retry_and_cleanup_task_handler
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
        eprintln!("Retry and Cleanup task failed: {:?}", e);
    }
    Ok(())
}
