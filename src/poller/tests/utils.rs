use super::*;

#[test]
fn test_extract_urls_from_text() {
    let empty_text = "";
    let url1 = "https://www.example.com";
    let url2 = "https://www.example2.com";
    let url_with_excluded_keyword_benign = "https://www.example.com/musicbrainz-something";
    let email = "yellowhatpro3119@gmail.com";
    let multiple_urls = format!("{url1} //.,<> best {url2}\\[] <> {email}");
    assert_eq!(extract_urls_from_text(empty_text), Vec::<String>::new());
    assert_eq!(
        extract_urls_from_text(url1),
        vec!["https://www.example.com".to_string()]
    );
    assert_eq!(
        extract_urls_from_text(multiple_urls.as_str()),
        vec![
            "https://www.example.com".to_string(),
            "https://www.example2.com".to_string()
        ]
    );
    assert_eq!(extract_urls_from_text(email), Vec::<String>::new());
    assert_eq!(
        extract_urls_from_text(url_with_excluded_keyword_benign),
        vec!["https://www.example.com/musicbrainz-something".to_string()]
    );
}

#[test]
fn test_should_exclude_url() {
    let url1 = "https://www.musicbrainz.org";
    let url2 = "https://www.metabrainz.org";
    assert!(should_exclude_url(url1));
    assert!(should_exclude_url(url2));
}

#[test]
fn test_extract_url_from_json_with_empty_json() {
    let json = json!({});
    assert_eq!(extract_urls_from_json(&json, 12), Vec::<String>::new());
}

#[test]
fn test_extract_url_from_json_with_no_urls_containing_edit() {
    let json = json!({"entity": {"id": 56583, "mbid": "113664a0-3109-42fc-a7a9-0c7473103673", "name": "Whatever Gets You Through the Day"},
            "cover_art_id": 18799571682u64, "cover_art_types": ["3"], "cover_art_comment": "", "cover_art_position": 7, "cover_art_mime_type": "image/jpeg"});
    assert_eq!(extract_urls_from_json(&json, 32), Vec::<String>::new());
}

#[test]
fn test_add_relationship_type0_url() {
    let json = json!({
       "edit_version" : 2,
       "ended" : 0,
       "entity0" : {
          "gid" : "cbb08c5e-a6f4-4bd2-b8ce-f83b3564bfa6",
          "id" : 3798456,
          "name" : "http://lyrics.wikia.com/wiki/Love_And_Rockets:Here_Comes_The_Comedown"
       },
       "entity1" : {
          "gid" : "0d7a3891-9406-3fc1-bb28-4ce66f9d9d9b",
          "id" : 1633784,
          "name" : "Here Come the Comedown"
       },
       "entity_id" : 108288,
       "link_type" : {
          "id" : 271,
          "link_phrase" : "lyrics page for",
          "long_link_phrase" : "{entity1} has lyrics available at {entity0}",
          "name" : "lyrics",
          "reverse_link_phrase" : "lyrics page"
       },
       "type0" : "url",
       "type1" : "work"
    });
    assert_eq!(
        extract_url_from_add_relationship(&json),
        Some("http://lyrics.wikia.com/wiki/Love_And_Rockets:Here_Comes_The_Comedown".to_string())
    )
}

#[test]
fn test_add_relationship_type1_url() {
    let json = json!({
       "edit_version" : 2,
       "ended" : 0,
       "entity0" : {
          "gid" : "2b33914b-dbdb-441a-97e9-20e154df326a",
          "id" : 2098912,
          "name" : "No molestes más"
       },
       "entity1" : {
          "gid" : "3760a8dd-317b-4915-9ade-805d98baabf1",
          "id" : 4758108,
          "name" : "https://www.deezer.com/album/49210932"
       },
       "entity_id" : 1946776,
       "link_type" : {
          "id" : 85,
          "link_phrase" : "stream {video} for free",
          "long_link_phrase" : "{video} can be streamed for free at",
          "name" : "streaming music",
          "reverse_link_phrase" : "free music {video} streaming page for"
       },
       "type0" : "release",
       "type1" : "url"
    });
    assert_eq!(
        extract_url_from_add_relationship(&json),
        Some("https://www.deezer.com/album/49210932".to_string())
    );
}

#[test]
fn test_edit_relationship_url() {
    let json_containing_no_url = json!({
       "edit_version" : 2,
       "entity0_credit" : "",
       "entity1_credit" : "",
       "link" : {
          "attributes" : [],
          "begin_date" : {
             "day" : null,
             "month" : null,
             "year" : null
          },
          "end_date" : {
             "day" : null,
             "month" : null,
             "year" : null
          },
          "ended" : 0,
          "entity0" : {
             "gid" : "28aee604-f120-4914-bd52-5bc500477a06",
             "id" : 84159,
             "name" : "Witness: The Tour: Québec City"
          },
          "entity1" : {
             "gid" : "fe469abf-6179-4ec7-8bda-4f375371d695",
             "id" : 25354,
             "name" : "Witness: The Tour"
          },
          "link_type" : {
             "id" : 802,
             "link_phrase" : "part of",
             "long_link_phrase" : "is a part of",
             "name" : "part of",
             "reverse_link_phrase" : "has parts"
          }
       },
       "new" : {
          "attributes" : [
             {
                "text_value" : "1.3",
                "type" : {
                   "gid" : "a59c5830-5ec7-38fe-9a21-c7ea54f6650a",
                   "id" : 788,
                   "name" : "number",
                   "root" : {
                      "gid" : "a59c5830-5ec7-38fe-9a21-c7ea54f6650a",
                      "id" : 788,
                      "name" : "number"
                   }
                }
             }
          ]
       },
       "old" : {
          "attributes" : []
       },
       "relationship_id" : 44610,
       "type0" : "event",
       "type1" : "series"
    }
    );

    let json_containing_url = json!({
       "edit_version" : 2,
       "entity0_credit" : "",
       "entity1_credit" : "",
       "link" : {
          "attributes" : [],
          "begin_date" : {
             "day" : null,
             "month" : null,
             "year" : null
          },
          "end_date" : {
             "day" : null,
             "month" : null,
             "year" : null
          },
          "ended" : 0,
          "entity0" : {
             "gid" : "679ea5f4-23b0-41ca-844f-8dfa3c497fcf",
             "id" : 1158366,
             "name" : "James McKenty"
          },
          "entity1" : {
             "gid" : "e094017b-1feb-4bce-bac7-ca151dd9c070",
             "id" : 2693721,
             "name" : "http://www.mykawartha.com/community-story/3714370-peterbio-james-mckenty/"
          },
          "link_type" : {
             "id" : 707,
             "link_phrase" : "interviews",
             "long_link_phrase" : "has an interview at",
             "name" : "interview",
             "reverse_link_phrase" : "interview with"
          }
       },
       "new" : {
          "entity1" : {
             "gid" : "fe748004-ef64-4b5d-a5ad-b230d7cb48cc",
             "id" : 12938147,
             "name" : "https://web.archive.org/web/20141002201130/http://www.mykawartha.com/community-story/3714370-peterbio-james-mckenty/"
          }
       },
       "old" : {
          "entity1" : {
             "gid" : "e094017b-1feb-4bce-bac7-ca151dd9c070",
             "id" : 2693721,
             "name" : "http://www.mykawartha.com/community-story/3714370-peterbio-james-mckenty/"
          }
       },
       "relationship_id" : 958424,
       "type0" : "artist",
       "type1" : "url"
    }
    );
    assert!(extract_url_from_edit_relationship(&json_containing_no_url).is_none());
    assert_eq!(extract_url_from_edit_relationship(&json_containing_url), Some("https://web.archive.org/web/20141002201130/http://www.mykawartha.com/community-story/3714370-peterbio-james-mckenty/".to_string()))
}

#[test]
fn test_any_annotations() {
    let series_annotation = json!({
       "annotation_id" : 1272483,
       "changelog" : null,
       "editor_id" : 2469102,
       "entity" : {
          "id" : 5286,
          "name" : "Film Themes \\ Great Film Themes \\ All the Best of Soundtracks"
       },
       "old_annotation_id" : 1086953,
       "text" : "This same series gets renamed many times.  At least four examples, likely more.  Always on a cheap reissue label.  The artist is usually missing or madeup.\n\nWIP: Adds a link to this conversation: https://musicbrainz.org/edit/28734798\n\nThat spotted a Countdown Music reference, but there are also some clear links to Russell B going on here too...\n\nFocus is kept on the AcoustIDs links here especically.  It is quite possible Countdown Music bought in some Russell B tracks, or Russell B is being credited to Countdown...  all a tangled mess in cheap compilation re-issue world.\n"
    });
    assert_eq!(
        extract_url_from_any_annotation(&series_annotation),
        Some(vec!["https://musicbrainz.org/edit/28734798".to_string()])
    );
}

#[sqlx::test(fixtures(
    "../../../tests/fixtures/InternetArchiveUrls.sql",
    "../../../tests/fixtures/EditNote.sql",
    "../../../tests/fixtures/EditData.sql",
    "../../../tests/fixtures/LastUnprocessedRows.sql"
))]
async fn test_get_edit_data_and_note_start_id(pool: PgPool) -> Result<(), Error> {
    let last_row = get_edit_data_and_note_start_id(&pool).await?;
    assert_eq!(last_row, (3, 3));
    sqlx::query("DELETE FROM external_url_archiver.internet_archive_urls;")
        .execute(&pool)
        .await
        .unwrap();
    let last_row_when_no_rows = get_edit_data_and_note_start_id(&pool).await?;
    assert_eq!(last_row_when_no_rows, (3, 3));
    Ok(())
}

#[sqlx::test(fixtures(
    "../../../tests/fixtures/InternetArchiveUrls.sql",
    "../../../tests/fixtures/EditNote.sql",
    "../../../tests/fixtures/EditData.sql",
    "../../../tests/fixtures/LastUnprocessedRows.sql"
))]
async fn test_get_edit_data_and_note_start_id_only_edit_data(pool: PgPool) -> Result<(), Error> {
    sqlx::query("DELETE FROM external_url_archiver.internet_archive_urls;")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(r#"
     INSERT INTO external_url_archiver.internet_archive_urls(url, from_table, from_table_id, retry_count)
     VALUES ('https://example.com', 'edit_data', 1000, 0);
    "#
    ).execute(&pool)
        .await
        .unwrap();
    let last_row_when_no_rows = get_edit_data_and_note_start_id(&pool).await?;
    assert_eq!(last_row_when_no_rows, (3, 3));
    Ok(())
}

#[sqlx::test(fixtures(
    "../../../tests/fixtures/InternetArchiveUrls.sql",
    "../../../tests/fixtures/EditNote.sql",
    "../../../tests/fixtures/EditData.sql",
    "../../../tests/fixtures/LastUnprocessedRows.sql"
))]
async fn test_get_edit_data_and_note_start_id_only_edit_note(pool: PgPool) -> Result<(), Error> {
    sqlx::query("DELETE FROM external_url_archiver.internet_archive_urls;")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(r#"
     INSERT INTO external_url_archiver.internet_archive_urls(url, from_table, from_table_id, retry_count)
     VALUES ('https://example.com', 'edit_note', 1000, 0);
    "#
    ).execute(&pool)
        .await.unwrap();
    let last_row_when_no_rows = get_edit_data_and_note_start_id(&pool).await?;
    assert_eq!(last_row_when_no_rows, (3, 3));
    Ok(())
}

#[sqlx::test(fixtures(
    "../../../tests/fixtures/Editor.sql",
    "../../../tests/fixtures/EditNote.sql"
))]
async fn test_extract_url_from_edit_note(pool: PgPool) -> Result<(), Error> {
    let note_with_no_url = sqlx::query_as::<_, EditNote>(
        r#"
            SELECT *
            FROM edit_note
            WHERE id = 1
        "#,
    )
    .fetch_one(&pool)
    .await?;
    let note_with_url = sqlx::query_as::<_, EditNote>(
        r#"
            SELECT *
            FROM edit_note
            WHERE id = 2
        "#,
    )
    .fetch_one(&pool)
    .await?;
    let note_with_edit_spammer = sqlx::query_as::<_, EditNote>(
        r#"
            SELECT *
            FROM edit_note
            WHERE id = 3
        "#,
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(
        extract_url_from_edit_note(&note_with_no_url, &pool).await,
        Vec::<String>::new()
    );
    assert_eq!(
        extract_url_from_edit_note(&note_with_edit_spammer, &pool).await,
        Vec::<String>::new()
    );
    assert_eq!(
        extract_url_from_edit_note(&note_with_url, &pool).await,
        vec!["https://example.com".to_string()]
    );
    Ok(())
}

#[sqlx::test(fixtures(
    "../../../tests/fixtures/Editor.sql",
    "../../../tests/fixtures/EditData.sql",
    "../../../tests/fixtures/Edit.sql"
))]
async fn test_extract_url_from_edit_data(pool: PgPool) -> Result<(), Error> {
    let edit_with_no_url = sqlx::query_as::<_, EditData>(
        r#"
            SELECT *
            FROM edit_data
            WHERE edit = 3
        "#,
    )
    .fetch_one(&pool)
    .await?;
    let edit_with_url = sqlx::query_as::<_, EditData>(
        r#"
            SELECT *
            FROM edit_data
            WHERE edit = 2
        "#,
    )
    .fetch_one(&pool)
    .await?;
    let edit_with_edit_spammer = sqlx::query_as::<_, EditData>(
        r#"
            SELECT *
            FROM edit_data
            WHERE edit = 1
        "#,
    )
    .fetch_one(&pool)
    .await?;
    assert_eq!(
        extract_url_from_edit_data(&edit_with_edit_spammer, &pool).await,
        Vec::<String>::new()
    );
    assert_eq!(
        extract_url_from_edit_data(&edit_with_no_url, &pool).await,
        Vec::<String>::new()
    );
    assert_eq!(
        extract_url_from_edit_data(&edit_with_url, &pool).await,
        vec!["https://www.discogs.com/artist/296705".to_string()]
    );
    Ok(())
}
