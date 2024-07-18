use crate::structs::internet_archive_urls::InternetArchiveUrls;
use linkify::{LinkFinder, LinkKind};
use mb_rs::schema::{EditData, EditNote};
use serde_json::{json, Value};
use sqlx::types::JsonValue;
use sqlx::{Error, PgPool};

/// This function takes input Edit Note, checks if
/// - the EditNote's text contains URL
/// - the Editor is not a spammer
///
/// and returns the vector of (URL as String)
pub async fn extract_url_from_edit_note(note: &EditNote, pool: &PgPool) -> Vec<String> {
    let editor = note.editor;
    match get_is_editor_spammer(editor, pool).await {
        Ok(false) => extract_urls_from_text(note.text.as_str()),
        _ => Vec::new(),
    }
}

/// This function takes input Edit Data, checks if
/// - the Edit Data contains URL
/// - the Editor is not a spammer
///
/// and returns the vector of (URL as String)
pub async fn extract_url_from_edit_data(edit: &EditData, pool: &PgPool) -> Vec<String> {
    let json: &Value = &edit.data;
    get_edit_type_if_editor_is_not_spammer(edit.edit, pool)
        .await
        .map(|edit_type| extract_urls_from_json(json, edit_type))
        .unwrap_or_default()
}

/// This function takes text and outputs a vector of URLs as string
pub fn extract_urls_from_text(text: &str) -> Vec<String> {
    let mut finder = LinkFinder::new();
    finder.kinds(&[LinkKind::Url]);

    let urls: Vec<_> = finder
        .links(text)
        .map(|link| link.as_str().to_string())
        .collect();
    urls
}

/// This function get the urls contained in the input edit data.
///
/// Depending on the type of edit data, it will give one or many urls:
///  - "Add" and "edit" edits will give 1 url, as the edit concern a 1:1 relation between an entity and an Url
///  - Annotations provide multiple Urls as it is arbitrary text made by the editor.
///
/// The input edit data is in [`JsonValue`] form.
pub fn extract_urls_from_json(json: &JsonValue, edit_type: i16) -> Vec<String> {
    match edit_type {
        90 => extract_url_from_add_relationship(json)
            .map(|url| vec![url])
            .unwrap_or_default(),
        91 => extract_url_from_edit_relationship(json)
            .map(|url| vec![url])
            .unwrap_or_default(),
        101 => extract_url_from_edit_url(json)
            .map(|url| vec![url])
            .unwrap_or_default(),
        _ => extract_url_from_any_annotation(json).unwrap_or_default(),
    }
}

fn extract_url_from_add_relationship(json: &JsonValue) -> Option<String> {
    if json.get("type0") == Some(&json!("url")) {
        if json.get("entity0").is_some() && json["entity0"].get("name").is_some() {
            let mut url = json["entity0"]["name"].to_string();
            url = url.replace(['\"', ' '], "");
            return Some(url);
        };
    } else if json.get("type1") == Some(&json!("url"))
        && json.get("entity1").is_some()
        && json["entity1"].get("name").is_some()
    {
        let mut url = json["entity1"]["name"].to_string();
        url = url.replace(['\"', ' '], "");
        return Some(url);
    }
    None
}

fn extract_url_from_edit_relationship(json: &JsonValue) -> Option<String> {
    if json.get("type0") == Some(&json!("url")) {
        if json.get("new").is_some()
            && json["new"].get("entity0").is_some()
            && json["new"]["entity0"].get("name").is_some()
        {
            let mut url = json["new"]["entity0"]["name"].to_string();
            url = url.replace(['\"', ' '], "");
            return Some(url);
        }
    } else if json.get("type1") == Some(&json!("url"))
        && json.get("new").is_some()
        && json["new"].get("entity1").is_some()
        && json["new"]["entity1"].get("name").is_some()
    {
        let mut url = json["new"]["entity1"]["name"].to_string();
        url = url.replace(['\"', ' '], "");
        return Some(url);
    };
    None
}

fn extract_url_from_edit_url(json: &JsonValue) -> Option<String> {
    if json.get("new").is_some() && json["new"].get("url").is_some() {
        let mut url = json["new"]["url"].to_string();
        url = url.replace(['\"', ' '], "");
        return Some(url);
    }
    None
}

fn extract_url_from_any_annotation(json: &JsonValue) -> Option<Vec<String>> {
    if json.get("text").is_some() {
        let result = extract_urls_from_text(json["text"].as_str().unwrap());
        if !result.is_empty() {
            return Some(result);
        };
    }
    None
}

/// Returns true if a editor is marked spammer
pub async fn get_is_editor_spammer(editor_id: i32, pool: &PgPool) -> Result<bool, Error> {
    let query = r#"
    SELECT privs & 4096 != 0 as is_editor_spammer
    FROM editor
    WHERE id = $1;
    "#;
    sqlx::query_as::<_, (bool,)>(query)
        .bind(editor_id)
        .fetch_one(pool)
        .await
        .map(|is_editor_spammer_row| is_editor_spammer_row.0)
}

/// Returns the edit type if the editor is not spammer, `None` the editor is spammer
pub async fn get_edit_type_if_editor_is_not_spammer(
    edit_id: i32,
    pool: &PgPool,
) -> Result<i16, Error> {
    let query = r#"
    SELECT edit.type
    FROM edit
    JOIN editor ON edit.editor = editor.id
    WHERE edit.id = $1
    AND (editor.privs & 4096) = 0;
    "#;

    sqlx::query_as::<_, (i16,)>(query)
        .bind(edit_id)
        .fetch_one(pool)
        .await
        .map(|edit_type_row| edit_type_row.0)
}

///This function fetches the latest row from internet_archive_urls_table
pub async fn get_edit_data_and_note_start_id(pool: &PgPool) -> (i32, i32) {
    let last_row = sqlx::query_as::<_, InternetArchiveUrls>(
        r#"
        SELECT DISTINCT ON (from_table)
        id, url, job_id, from_table, from_table_id, created_at, retry_count, is_saved
        FROM external_url_archiver.internet_archive_urls
        WHERE from_table IN ('edit_data', 'edit_note')
        ORDER BY from_table, from_table_id DESC;
        "#,
    )
    .fetch_all(pool)
    .await;
    return match last_row {
        Ok(res) => match res.len() {
            1 => {
                if res.first().unwrap().from_table == Some("edit_data".to_string()) {
                    let edit_data_id = res.first().unwrap().from_table_id.unwrap();
                    let edit_note_id = get_latest_edit_note_id(pool).await;
                    (edit_data_id, edit_note_id)
                } else {
                    let edit_note_id = res.first().unwrap().from_table_id.unwrap();
                    let edit_data_id = get_latest_edit_data_id(pool).await;
                    (edit_data_id, edit_note_id)
                }
            }
            2 => (res[0].from_table_id.unwrap(), res[1].from_table_id.unwrap()),
            _ => initialise_empty_internet_archive_table(pool).await,
        },
        Err(e) => {
            eprintln!(
                "Error fetching edit data and edit note start id to start polling with. Error: {}",
                e
            );
            initialise_empty_internet_archive_table(pool).await
        }
    };
}

//TODO: Make the following logic better!
///This function should run when there is no internet_archive_urls table or the table is not populated
pub async fn initialise_empty_internet_archive_table(pool: &PgPool) -> (i32, i32) {
    let latest_edit_note = get_latest_edit_note_id(pool).await;
    let latest_edit = get_latest_edit_data_id(pool).await;
    println!("Latest edit: {}, note: {}", latest_edit, latest_edit_note);
    // 0th-> Edit Data, 1st -> Edit Note
    //TODO: Uncomment it later
    // (latest_edit, latest_edit_note)
    (111451813, 71025805)
}

pub async fn get_latest_edit_data_id(pool: &PgPool) -> i32 {
    let select_latest_edit_data_row = "
         SELECT DISTINCT ON (edit)
         *
         FROM edit_data
         ORDER BY edit  DESC limit 1";

    let latest_edit_data_row = sqlx::query_as::<_, EditData>(select_latest_edit_data_row)
        .fetch_one(pool)
        .await;
    latest_edit_data_row.unwrap().edit
}

pub async fn get_latest_edit_note_id(pool: &PgPool) -> i32 {
    let select_latest_edit_note_row = "
         SELECT DISTINCT ON (id)
         *
         FROM edit_note
         ORDER BY id  DESC LIMIT 1";

    let latest_edit_note_row = sqlx::query_as::<_, EditNote>(select_latest_edit_note_row)
        .fetch_one(pool)
        .await;
    latest_edit_note_row.unwrap().id
}

/// This function takes input a URL string, and returns true if it should exclude the URL from saving
pub fn should_exclude_url(url: &str) -> bool {
    // TODO: discuss and add keywords to identify URLs we want to exclude
    let keywords: Vec<&str> = vec![
        "www.musicbrainz.org",
        "https://musicbrainz.org",
        "www.metabrainz.org",
        "https://metabrainz.org",
        "web.archive.org",
    ];
    keywords.iter().any(|keyword| url.contains(keyword))
}

///This function checks if we are inserting the same url within a day into the internet_archive_urls table
pub async fn should_insert_url_to_internet_archive_urls(
    url: &str,
    pool: &PgPool,
) -> Result<bool, Error> {
    if should_exclude_url(url) {
        return Ok(false);
    }
    let res: Option<(bool,)> = sqlx::query_as(
        r#"
        SELECT (CURRENT_TIMESTAMP - created_at) > INTERVAL '1 DAY' AS daydiff
        FROM external_url_archiver.internet_archive_urls
        WHERE url = $1
        "#,
    )
    .bind(url)
    .fetch_optional(pool)
    .await?;
    if res.is_some() {
        let (bool_val,) = res.unwrap();
        Ok(bool_val)
    } else {
        Ok(true)
    }
}

pub async fn save_url_to_internet_archive_urls(
    url: &str,
    from_table: &str,
    from_table_id: i32,
    pool: &PgPool,
) -> Result<(), Error> {
    if !should_insert_url_to_internet_archive_urls(url, pool).await? {
        return Ok(());
    }
    let query = r#"
                    INSERT INTO external_url_archiver.internet_archive_urls (url, from_table, from_table_id, retry_count, is_saved)
                    VALUES ($1, $2, $3, 0, false)"#;
    sqlx::query(query)
        .bind(url)
        .bind(from_table)
        .bind(from_table_id)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
#[path = "./tests/utils.rs"]
mod tests;
