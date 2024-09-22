use super::*;
use crate::configuration::SETTINGS;
use sqlx::Error;

#[sqlx::test(fixtures(
    "../../../tests/fixtures/InternetArchiveUrls.sql",
    "../../../tests/fixtures/internet_archive_urls_dump.sql"
))]
async fn test_get_first_index_to_start_notifier_from(pool: PgPool) -> Result<(), Error> {
    let first_index_to_start_notifier_from =
        get_first_id_to_start_notifier_from(pool.clone()).await;
    assert_eq!(first_index_to_start_notifier_from.unwrap(), 12);
    sqlx::query(
        r#"
            DELETE FROM external_url_archiver.internet_archive_urls
            WHERE id = 12;
            "#,
    )
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(
        get_first_id_to_start_notifier_from(pool.clone())
            .await
            .unwrap(),
        13
    );
    Ok(())
}

#[tokio::test]
async fn test_make_archival_network_request() -> Result<(), ArchivalError> {
    let testing_url_invalid = "www.example.om";
    let opts = mockito::ServerOpts {
        host: "127.0.0.1",
        port: 1234,
        ..Default::default()
    };
    let mut server = mockito::Server::new_with_opts_async(opts).await;
    let mock = server
        .mock("POST", "/save")
        .match_header("Accept", "application/json")
        .match_header("Authorization", format!("LOW {}:{}", SETTINGS.wayback_machine_api.myaccesskey, SETTINGS.wayback_machine_api.mysecret).as_str())
        .match_header("Content-Type", "application/x-www-form-urlencoded")
        .match_body(format!("url={}", testing_url_invalid).as_str())
        .with_body(r#"{"message":"www.example.om URL syntax is not valid.","status":"error","status_ext":"error:invalid-url-syntax"}"#)
        .create();

    let testing_url = "www.example.com";
    let mock2 = server
        .mock("POST", "/save")
        .match_header("Accept", "application/json")
        .match_header(
            "Authorization",
            format!(
                "LOW {}:{}",
                SETTINGS.wayback_machine_api.myaccesskey, SETTINGS.wayback_machine_api.mysecret
            )
            .as_str(),
        )
        .match_header("Content-Type", "application/x-www-form-urlencoded")
        .match_body(format!("url={}", testing_url).as_str())
        .with_body(r#"html response here"#)
        .create();

    let mock3 = server
        .mock("POST", "/save")
        .match_header("Accept", "application/json")
        .match_header(
            "Authorization",
            format!(
                "LOW {}:{}",
                SETTINGS.wayback_machine_api.myaccesskey, SETTINGS.wayback_machine_api.mysecret
            )
            .as_str(),
        )
        .match_header("Content-Type", "application/x-www-form-urlencoded")
        .match_body(format!("url={}", testing_url).as_str())
        .with_body(r#"{"url": "www.example.com", "job_id": "12345" }"#)
        .create();
    let response = make_archival_network_request("www.example.om").await;
    assert!(response.is_err());
    mock.assert();
    let response2 = make_archival_network_request("www.example.com").await;
    assert!(response2.is_err());
    mock2.assert();
    let response3 = make_archival_network_request("www.example.com").await;
    assert!(response3.is_ok());
    mock3.assert();

    Ok(())
}
