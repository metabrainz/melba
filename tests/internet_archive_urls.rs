#[cfg(test)]
mod internet_archive_urls_tests {
    use serde::Deserialize;
    use sqlx::{PgPool, query, query_as};

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
