use mb_exurl_ia_service::archival;
use mb_exurl_ia_service::archival::error::ArchivalError;
use mb_exurl_ia_service::archival::listener::handle_payload;
use mb_exurl_ia_service::archival::notifier::Notifier;
use mb_exurl_ia_service::structs::internet_archive_urls::InternetArchiveUrls;
use sqlx::postgres::PgListener;
use sqlx::{Error, PgPool};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time;
use tokio::time::Instant;

#[sqlx::test(fixtures(
    "../fixtures/InternetArchiveUrls.sql",
    "../fixtures/internet_archive_urls_dump.sql"
))]
async fn test_notifier(pool: PgPool) -> Result<(), Error> {
    let mut notifier = Notifier::new(pool.clone()).await;
    let mut current_id = 12;
    let end_id = 70;
    for _ in 0..100 {
        if notifier.should_notify().await {
            if current_id <= end_id {
                assert_eq!(notifier._get_notifier_index(), current_id);
            }
            notifier.notify().await.unwrap();
            current_id += 1;
            if current_id <= end_id {
                assert_eq!(notifier._get_notifier_index(), current_id);
            }
        }
    }
    Ok(())
}

#[sqlx::test(fixtures(
    "../fixtures/InternetArchiveUrls.sql",
    "../fixtures/internet_archive_urls_dump.sql"
))]
async fn test_archival(pool: PgPool) -> Result<(), ArchivalError> {
    let notifier = Arc::new(Mutex::new(Notifier::new(pool.clone()).await));
    let listener_pool = pool.clone();

    // Spawn both tasks and use tokio::join! to run them concurrently
    let (notifier_result, listener_result) = tokio::join!(
        // Notifier task
        tokio::spawn({
            let notifier = Arc::clone(&notifier);
            async move {
                for _ in 0..40 {
                    let mut notifier = notifier.lock().await;
                    if notifier.should_notify().await {
                        notifier.notify().await.unwrap();
                        println!("{}", notifier._get_notifier_index());
                    }
                }
            }
        }),
        // Listener task
        tokio::spawn(async move {
            let mut listener = PgListener::connect_with(&listener_pool).await.unwrap();
            listener.listen("archive_urls").await.unwrap();

            let duration = Duration::from_secs(5 * 60);
            let start_time = Instant::now();

            // Loop until the specified duration has elapsed
            while start_time.elapsed() < duration {
                if let Some(notification) = listener.try_recv().await.unwrap() {
                    let payload: InternetArchiveUrls =
                        serde_json::from_str(notification.payload()).unwrap();
                    assert!(payload.url.is_some());
                    handle_payload(payload, &listener_pool).await.unwrap();
                }

                // Sleep for 5 seconds between checks
                time::sleep(Duration::from_secs(5)).await;
            }

            println!("Listener has run for 5 minutes and will now stop.");
        })
    );

    // Check both results for errors
    notifier_result.unwrap();
    listener_result.unwrap();

    Ok(())
}

#[sqlx::test(fixtures(
    "../fixtures/InternetArchiveUrls.sql",
    "../fixtures/internet_archive_urls_dump.sql"
))]
async fn test_cleanup_task(pool: PgPool) -> Result<(), ArchivalError> {
    archival::retry::start(pool.clone()).await.unwrap();
    let success_urls = sqlx::query_as::<_, InternetArchiveUrls>(
        r#"
            SELECT * FROM external_url_archiver.internet_archive_urls
            WHERE status = 3;
            "#,
    )
    .fetch_all(&pool)
    .await?;
    //Check cleanup
    assert_eq!(success_urls.len(), 0);
    Ok(())
}
