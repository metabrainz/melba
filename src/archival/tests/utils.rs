use super::*;
use sqlx::Error;

#[sqlx::test(fixtures("../../../tests/fixtures/InternetArchiveUrls.sql"))]
async fn test_get_first_index_to_start_notifier_from(pool: PgPool) -> Result<(), Error> {
    let first_index_to_start_notifier_from =
        get_first_id_to_start_notifier_from(pool.clone()).await;
    assert_eq!(first_index_to_start_notifier_from.unwrap(), 1);
    sqlx::query(
        r#"
            DELETE FROM external_url_archiver.internet_archive_urls
            WHERE id = 1;
            "#,
    )
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(
        get_first_id_to_start_notifier_from(pool.clone())
            .await
            .unwrap(),
        2
    );
    Ok(())
}

#[sqlx::test(fixtures("../../../tests/fixtures/InternetArchiveUrls.sql"))]
async fn test_update_internet_archive_urls(pool: PgPool) -> Result<(), Error> {
    update_internet_archive_urls_with_job_id(&pool, "123abc".to_string(), 4).await?;
    let updated_res = sqlx::query_as::<_, InternetArchiveUrls>(
        r#"
        SELECT * FROM external_url_archiver.internet_archive_urls
        WHERE id = $1
        "#,
    )
    .bind(4)
    .fetch_one(&pool)
    .await;

    if let Ok(res) = updated_res {
        assert_eq!(res.is_saved.unwrap(), true);
        assert_eq!(res.job_id.unwrap(), "123abc");
    } else {
        panic!("Should return row")
    }
    Ok(())
}
