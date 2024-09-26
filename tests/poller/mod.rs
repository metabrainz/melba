use melba::poller::looper::poll_db;
use melba::poller::utils::get_edit_data_and_note_start_id;
use melba::structs::internet_archive_urls::InternetArchiveUrls;
use sqlx::PgPool;

#[sqlx::test(fixtures(
    "../fixtures/InternetArchiveUrls.sql",
    "../fixtures/LastUnprocessedRows.sql",
    "../fixtures/last_unprocessed_rows_dump.sql",
    "../fixtures/Editor.sql",
    "../fixtures/editor_dump.sql",
    "../fixtures/EditNote.sql",
    "../fixtures/edit_note_dump.sql",
    "../fixtures/EditData.sql",
    "../fixtures/edit_data_dump.sql",
    "../fixtures/Edit.sql",
    "../fixtures/edit_dump.sql"
))]
async fn test_poller(pool: PgPool) -> Result<(), sqlx::Error> {
    let (mut edit_data_start_idx, mut edit_note_start_idx) =
        get_edit_data_and_note_start_id(&pool).await?;
    {
        // On polling 10 rows from start of edit_data and edit_note (poll_db polls 10 rows each time)
        assert_eq!(edit_data_start_idx, 111450838);
        assert_eq!(edit_note_start_idx, 71024901);
        let (edit_data_new_row, edit_note_new_row) =
            poll_db(&pool, edit_data_start_idx, edit_note_start_idx).await?;
        let query = r#"
        SELECT *
        FROM external_url_archiver.internet_archive_urls"#
            .to_string();
        let rows = sqlx::query_as::<_, InternetArchiveUrls>(&query)
            .fetch_all(&pool)
            .await?;
        let first_row = rows.first().unwrap();
        let rows_len = rows.len();
        assert_eq!(rows_len, 1);
        assert_eq!(first_row.url, Some("https://www.jazzdisco.org/verve-records/catalog-folk-blues-3000-4000-series/#mgv-4006-2".to_string()));
        assert_eq!(edit_data_new_row, Some(111450848));
        assert_eq!(edit_note_new_row, Some(71024911));
        edit_data_start_idx = edit_data_new_row.unwrap();
        edit_note_start_idx = edit_note_new_row.unwrap();
    }
    {
        // Polling 50 more rows each from edit_data and edit_note
        for _ in 0..5 {
            let (edit_data_new_row, edit_note_new_row) =
                poll_db(&pool, edit_data_start_idx, edit_note_start_idx).await?;
            edit_data_start_idx = edit_data_new_row.unwrap();
            edit_note_start_idx = edit_note_new_row.unwrap();
        }
        assert_eq!(edit_data_start_idx, 111450898);
        assert_eq!(edit_note_start_idx, 71024961);
    }
    Ok(())
}
