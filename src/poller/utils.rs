use crate::poller::edit_types::remove_relationship::RemoveRelationship;
use crate::structs::last_unprocessed_row::LastUnprocessedRow;
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

    finder
        .links(text)
        .map(|link| link.as_str().to_string())
        .collect()
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
        92 => extract_url_from_remove_relationship(json)
            .map(|url| vec![url])
            .unwrap_or_default(),
        101 => extract_url_from_edit_url(json)
            .map(|url| vec![url])
            .unwrap_or_default(),
        _ => extract_url_from_any_annotation(json).unwrap_or_default(),
    }
}

fn extract_url_from_remove_relationship(json: &JsonValue) -> Option<String> {
    let remove_relationship_option: Option<RemoveRelationship> =
        serde_json::from_value(json.clone()).ok();

    if let Some(remove_relationship) = remove_relationship_option {
        if let Some(relationship) = remove_relationship.relationship {
            if let Some(link) = relationship.link {
                if let Some(type_field) = link.type_field {
                    if type_field.entity0_type == Some("url".to_string()) {
                        return relationship.entity0.and_then(|e| e.name);
                    } else if type_field.entity1_type == Some("url".to_string()) {
                        return relationship.entity1.and_then(|e| e.name);
                    }
                }
            }
        }
    }
    None
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
    let text = json.get("text");
    if text.is_some() && !text.unwrap().is_null() {
        let result = extract_urls_from_text(json.get("text").unwrap().as_str().unwrap());
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
pub async fn get_edit_data_and_note_start_id(pool: &PgPool) -> Result<(i32, i32), Error> {
    sqlx::query_as::<_, LastUnprocessedRow>(
        r#"
            SELECT * FROM external_url_archiver.last_unprocessed_rows
            WHERE table_name IN ('edit_data', 'edit_note');
            "#,
    )
    .fetch_all(pool)
    .await
    .map(|edit_data_and_edit_column| {
        (
            edit_data_and_edit_column[0].id_column,
            edit_data_and_edit_column[1].id_column,
        )
    })
}

///This function checks if the URL is already present in `internet_archive_urls` table
pub async fn should_insert_url_to_internet_archive_urls(
    url: &str,
    pool: &PgPool,
) -> Result<bool, Error> {
    if should_exclude_url(url) {
        return Ok(false);
    }

    match is_url_exists(url, pool).await {
        Ok(exists) => Ok(!exists), // If the URL exists, return false; otherwise, return true.
        Err(_) => Ok(true),        // On error, default to true
    }
}

pub async fn is_url_exists(url: &str, pool: &PgPool) -> Result<bool, Error> {
    let res: Option<(bool,)> = sqlx::query_as(
        r#"
        SELECT EXISTS (
            SELECT 1 as present
            FROM external_url_archiver.internet_archive_urls
            WHERE url = $1
        );
        "#,
    )
    .bind(url)
    .fetch_optional(pool)
    .await?;
    if res.is_some() {
        let bool_val = res.unwrap().0;
        Ok(bool_val)
    } else {
        Ok(false)
    }
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

/// Save the url to the `internet_archive_urls` table. Returns `Ok(true)` if the url has been sucessfully inserted.
/// Return `Ok(false)` if the url shouldn't be inserted
pub async fn save_url_to_internet_archive_urls(
    url: &str,
    from_table: &str,
    from_table_id: i32,
    pool: &PgPool,
) -> Result<bool, Error> {
    if !should_insert_url_to_internet_archive_urls(url, pool).await? {
        return Ok(false);
    }

    let query = r#"
                    INSERT INTO external_url_archiver.internet_archive_urls (url, from_table, from_table_id, retry_count)
                    VALUES ($1, $2, $3, 0)"#;

    sqlx::query(query)
        .bind(url)
        .bind(from_table)
        .bind(from_table_id)
        .execute(pool)
        .await?;

    Ok(true)
}

/// Update the latest processed id in `last_processed_rows` table
pub async fn update_last_unprocessed_rows(
    from_table: &str,
    table_id: i32,
    pool: &PgPool,
) -> Result<(), Error> {
    let query = r#"
        UPDATE external_url_archiver.last_unprocessed_rows 
        SET
        id_column = $1
        WHERE table_name = $2
    "#;

    sqlx::query(query)
        .bind(table_id)
        .bind(from_table)
        .execute(pool)
        .await?;

    Ok(())
}

#[cfg(test)]
#[path = "./tests/utils.rs"]
mod tests;
