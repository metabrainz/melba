use linkify::{LinkFinder, LinkKind};
use mb_rs::schema::EditData;
use serde_json::json;
use sqlx::Error;
use sqlx::types::JsonValue;
use crate::poller::internet_archive_urls::InternetArchiveUrls;

/// This function takes text from edit note and outputs a vector of URLs as string
pub fn extract_urls_from_edit_note(note: &str) -> Vec<String> {
    let mut finder = LinkFinder::new();
    finder.kinds(&[LinkKind::Url]);

    let mut urls: Vec<_> = finder
        .links(note)
        .map(|link|{link.as_str().to_string()})
        .collect();
    urls
}

/// This function takes input a URL string, and returns true if it should exclude the URL from saving
pub fn should_exclude_url(url: &str) -> bool {
    // TODO: discuss and add keywords to identify URLs we want to exclude
    let keywords: Vec<String> = vec![];
    keywords.iter().any(|keyword| url.contains(keyword))
}

/// This function takes input Edit Data in form of JSONValue, checks if the Edit Data contains URL, and returns the URL as String
pub fn extract_url_from_edit_data(json: JsonValue) -> Option<String> {
    return if json.get("type1") == Some(&json!("url")) {
        // Edit type: Add Relationship
        if json.get("entity1").is_none() {
            return None;
        }
        let entity0 = json.get("entity1").unwrap();
        Some(entity0.get("name").unwrap().to_string())
    } else if json.get("new").is_some() && json.get("new").unwrap().is_object() {
        //Edit type: Edit URL
        let new = json.get("new").unwrap();
        return if new.get("url").is_some() && new.get("url") != Some(&json!(null)) {
            Some(new.get("url").unwrap().to_string())
        } else { None }
    } else {
        None
    }
}

//TODO: Handle the cases: 1. Can we/should we retrieve latest rows faster?  2. Handle case when the internet_archive_urls table is empty
///This function fetches the latest row from internet_archive_urls_table
pub async fn extract_last_row_from_internet_archive_table(
    pool: &sqlx::PgPool
) -> Vec<InternetArchiveUrls> {
    let last_row = sqlx::query_as::<_, InternetArchiveUrls>(
        "
        SELECT DISTINCT ON (from_table)
        id, url, job_id, from_table, from_table_id, created_at, retry_count, is_saved
        FROM internet_archive_urls
        WHERE from_table IN ('edit_data', 'edit_note')
        ORDER BY from_table, from_table_id DESC;
        "
    )
        .fetch_all(pool)
        .await;
   return last_row.unwrap()
}