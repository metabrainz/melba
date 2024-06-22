#[cfg(test)]
mod internet_archive_urls_tests {
    use std::time::Duration;
    use serde::Deserialize;
    use sqlx::{PgPool, query, query_as};
    use sqlx::__rt::timeout;
    use sqlx::postgres::PgListener;

    #[derive(sqlx::FromRow, Debug, Deserialize)]
    pub struct InternetArchiveUrls {
        pub id: i32,
        pub url: Option<String>,
        pub job_id: Option<String>,
        pub from_table: Option<String>,
        pub from_table_id: Option<i32>,
        pub created_at: Option<chrono::DateTime<chrono::Utc>>,
        pub retry_count: Option<i32>,
        pub is_saved: Option<bool>,
    }

    #[sqlx::test(fixtures("InternetArchiveUrls"))]
    async fn test_select_from_table(pool: PgPool) -> sqlx::Result<()> {
        let row = query_as::<_, InternetArchiveUrls>(
            r#"SELECT *
                   FROM internet_archive_urls
                   WHERE from_table_id = 70000000;
                   "#
        ).fetch_one(&pool)
            .await.unwrap();
        assert_eq!(row.url.unwrap(), "https://blackpaintingsdiscography.bndcamp.com/album/asmodea");
        Ok(())
    }

    #[sqlx::test(fixtures("InternetArchiveUrls"))]
    async fn insert_into_table(pool: PgPool) -> sqlx::Result<()> {
        let query = r#"
        INSERT INTO internet_archive_urls (url, from_table, from_table_id, retry_count, is_saved) VALUES
        ('https://example.com', 'edit_note', 70000001, 0, false);"#;
        sqlx::query(query)
            .execute(&pool)
            .await?;
        let row = query_as::<_, InternetArchiveUrls>(
            r#"
                SELECT *
                FROM internet_archive_urls
                WHERE id = 5;
            "#
        ).fetch_one(&pool)
            .await?;
        assert_eq!(row.id, 5);
        assert_eq!(row.url.unwrap(), "https://example.com");
        assert_eq!(row.from_table.unwrap(), "edit_note");
        Ok(())
    }

    #[sqlx::test(fixtures("InternetArchiveUrls"))]
    async fn delete_from_table(pool: PgPool) -> sqlx::Result<()> {
        let query = r#"
            DELETE FROM internet_archive_urls
            WHERE from_table_id = 70000000;
        "#;
        sqlx::query(query)
            .execute(&pool)
            .await?;
        let row = query_as::<_, InternetArchiveUrls>(
            r#"
                SELECT *
                FROM internet_archive_urls
                WHERE from_table_id = 70000000;
            "#
        ).fetch_one(&pool)
            .await;
        assert!(row.is_err());
        Ok(())
    }

    #[sqlx::test(fixtures("InternetArchiveUrls"))]
    async fn test_pg_notify(pool: PgPool) -> sqlx::Result<()> {
        let query = r#"
            SELECT notify_archive_urls(1);
        "#;
        let mut listener = PgListener::connect_with(&pool).await?;
        listener.listen("archive_urls").await.unwrap();
        let handle = tokio::spawn(async move {
            let result = sqlx::query_as::<_, (i32, )>(query)
                .fetch_one(&pool)
                .await;
            match result {
                Ok(ans) => {
                    println!("Function returned: {:?}", ans);
                }
                Err(e) => {
                    println!("Error running query: {:?}", e);
                    return Err(e);
                }
            }
            Ok::<_, sqlx::Error>(())
        });

        let mut counter = 0usize;

        while counter < 2 {
            match timeout(Duration::from_secs(5), listener.recv()).await {
                Ok(Ok(notification)) => {
                    println!("Notification Payload: {}", notification.payload());
                    let payload: InternetArchiveUrls = serde_json::from_str(notification.payload()).unwrap();
                    println!("Received id: {}", payload.id);
                    assert!(!payload.url.unwrap().is_empty());
                    counter += 1;
                }
                Ok(Err(e)) => {
                    println!("Error receiving notification: {:?}", e);
                    break;
                }
                Err(_) => {
                    println!("Timeout waiting for notification");
                    break;
                }
            }
        }

        handle.await.unwrap()?;
        Ok(())
    }

    #[sqlx::test(fixtures("InternetArchiveUrls"))]
    async fn update_row_in_table(pool: PgPool) -> sqlx::Result<()> {
        query(
            r#"UPDATE internet_archive_urls
                   SET is_saved = true
                   WHERE from_table_id = 70000000;
                   "#
        )
            .execute(&pool)
            .await.
            unwrap();

        let row: InternetArchiveUrls = query_as::<_, InternetArchiveUrls>(
            r#"SELECT *
                   FROM internet_archive_urls
                   WHERE from_table_id = 70000000;
                   "#
        ).fetch_one(&pool)
            .await.unwrap();
        assert_eq!(row.is_saved.unwrap(), true);
        Ok(())
    }
}
