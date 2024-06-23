use std::sync::{Arc};
use std::time::Duration;
use sqlx::PgPool;
use tokio::join;
use tokio::sync::Mutex;
use crate::archival;
use crate::archival::notifier::Notifier;
use crate::poller::Poller;


pub async fn start(pool: &PgPool) -> Result<(), sqlx::Error> {

    const POLL_INTERVAL: u64 = 10;

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
                if let Err(e) = notifier.lock().await.notify().await {
                    println!("Notify failed, error: {}", e)
                };
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
