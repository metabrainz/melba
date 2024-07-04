use linkify::{LinkFinder, LinkKind};
use mb_rs::schema::{EditData, EditNote};
use serde_json::{json, Value};
use sqlx::{Error, PgPool};
use sqlx::types::JsonValue;
use crate::structs::internet_archive_urls::InternetArchiveUrls;

/// This function takes input Edit Note, checks if
/// - the EditNote's text contains URL
/// - the Editor is not a spammer
///
/// and returns the vector of (URL as String)
pub async fn extract_url_from_edit_note(note:&EditNote, pool: &PgPool) -> Vec<String> {
    let editor = note.editor;
    if get_is_editor_spammer(editor, pool).await {
        return vec![]
    }
    extract_urls_from_text(note.text.as_str())
}

/// This function takes input Edit Data, checks if
/// - the Edit Data contains URL
/// - the Editor is not a spammer
///
/// and returns the vector of (URL as String)
pub async fn extract_url_from_edit_data(edit: &EditData, pool: &PgPool) -> Vec<String> {
    let json: &Value = &edit.data;
    let editor = get_editor_id_from_edit_id(edit.edit, pool).await;
    if editor.is_ok() {
        if get_is_editor_spammer(editor.unwrap(), pool).await {
            return vec![]
        }
        extract_urls_from_json(json)
    } else {
        vec![]
    }
}

/// This function takes text and outputs a vector of URLs as string
pub fn extract_urls_from_text(text: &str) -> Vec<String> {
    let mut finder = LinkFinder::new();
    finder.kinds(&[LinkKind::Url]);

    let urls: Vec<_> = finder
        .links(text)
        .map(|link|{link.as_str().to_string()})
        .collect();
    urls
}

/// This function takes json and outputs a vector of URL as string
pub fn extract_urls_from_json(json: &JsonValue) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    if add_relationship_type0_url(&json).is_some() {
        result.push(add_relationship_type0_url(&json).unwrap());
    } else if add_relationship_type1_url(&json).is_some() {
        result.push(add_relationship_type1_url(&json).unwrap());
    } else if edit_relationship_type0_url(&json).is_some() {
        result.push(edit_relationship_type0_url(&json).unwrap());
    } else if edit_relationship_type1_url(&json).is_some() {
        result.push(edit_relationship_type1_url(&json).unwrap());
    } else if edit_url(&json).is_some() {
        result.push(edit_url(&json).unwrap());
    } else if any_annotation(&json).is_some() {
        result.append(&mut any_annotation(&json).unwrap());
    }
    result
}

fn add_relationship_type1_url(json: &JsonValue) -> Option<String> {
    if json.get("type1") == Some(&json!("url")) {
        if json.get("entity1").is_some() &&
            json["entity1"].get("name").is_some() {
            let mut url = json["entity1"]["name"].to_string();
            url = url.replace("\"", "").replace(" ", "");
            return Some(url)
        };
    }
    return None;
}

fn add_relationship_type0_url(json: &JsonValue) -> Option<String> {
    if json.get("type0") == Some(&json!("url")) {
        if json.get("entity0").is_some() &&
            json["entity0"].get("name").is_some() {
            let mut url = json["entity0"]["name"].to_string();
            url = url.replace("\"", "").replace(" ", "");
            return Some(url);
        };
    }
    return None;
}

fn edit_relationship_type0_url(json: &JsonValue) -> Option<String> {
    if json.get("type0") == Some(&json!("url")) {
        if json.get("new").is_some()
            && json["new"].get("entity0").is_some() &&
            json["new"]["entity0"].get("name").is_some() {
            let mut url = json["new"]["entity0"]["name"].to_string();
            url = url.replace("\"", "").replace(" ", "");
            return Some(url);
        };
    };
    return None;
}

fn edit_relationship_type1_url(json: &JsonValue) -> Option<String> {
    if json.get("type1") == Some(&json!("url")) {
        if json.get("new").is_some()
            && json["new"].get("entity1").is_some() &&
            json["new"]["entity1"].get("name").is_some() {
            let mut url = json["new"]["entity1"]["name"].to_string();
            url = url.replace("\"", "").replace(" ", "");
            return Some(url);
        };
    };
    return None;
}

fn edit_url(json: &JsonValue) -> Option<String> {
    if json.get("new").is_some() &&
        json["new"].get("url").is_some() {
        let mut url = json["new"]["url"].to_string();
        url = url.replace("\"", "").replace(" ", "");
        return Some(url);
    }
    return None;
}

fn any_annotation(json: &JsonValue) -> Option<Vec<String>> {
    if json.get("text").is_some() {
        let result = extract_urls_from_text(json["text"].as_str().unwrap());
        if !result.is_empty() {
            return Some(result)
        };
    }
    return None;
}

/// Returns true if a editor is marked spammer
pub async fn get_is_editor_spammer(
    editor_id: i32,
    pool: &PgPool
) -> bool {
    let query = r#"
    SELECT privs & 4096 != 0 as is_editor_spammer
    FROM editor
    WHERE id = $1;
    "#;
    let (is_editor_spammer, ) = sqlx::query_as::<_, (bool, )>(query)
        .bind(editor_id)
        .fetch_one(pool)
        .await
        .unwrap();
    return is_editor_spammer
}

///Returns the id of the editor, if edit id is given
pub async fn get_editor_id_from_edit_id(
    edit: i32,
    pool: &PgPool
) -> Result<i32, Error> {
    let editor_id = sqlx::query_as::<_, (i32, )>(
        r#"
               SELECT editor
               FROM edit
               WHERE id = $1;
            "#
    ).bind(edit)
        .fetch_one(pool)
        .await
        .map(|x| {
            return x.0
        });
    editor_id
}

///This function fetches the latest row from internet_archive_urls_table
pub async fn get_edit_data_and_note_start_id(
    pool: &PgPool
) -> (i32,i32) {
    let last_row = sqlx::query_as::<_, InternetArchiveUrls>(
        r#"
        SELECT DISTINCT ON (from_table)
        id, url, job_id, from_table, from_table_id, created_at, retry_count, is_saved
        FROM external_url_archiver.internet_archive_urls
        WHERE from_table IN ('edit_data', 'edit_note')
        ORDER BY from_table, from_table_id DESC;
        "#
    ).fetch_all(pool)
        .await;
    return match last_row {
        Ok(res) => {
            match res.len() {
                1 => {
                    if res.get(0).unwrap().from_table == Some("edit_data".to_string()) {
                        let edit_data_id = res.get(0).unwrap().from_table_id.unwrap();
                        let edit_note_id = get_latest_edit_note_id(pool).await;
                        (edit_data_id, edit_note_id)
                    } else {
                        let edit_note_id = res.get(0).unwrap().from_table_id.unwrap();
                        let edit_data_id = get_latest_edit_data_id(pool).await;
                        (edit_data_id, edit_note_id)
                    }
                },
                2 => {
                    (res[0].from_table_id.unwrap(), res[1].from_table_id.unwrap())
                },
                _ => {
                    initialise_empty_internet_archive_table(pool).await
                }
            }
        },
        Err(e) => {
            eprintln!("Error fetching edit data and edit note start id to start polling with. Error: {}", e);
            initialise_empty_internet_archive_table(pool).await
        }
    }
}

//TODO: Make the following logic better!
///This function should run when there is no internet_archive_urls table or the table is not populated
pub async fn initialise_empty_internet_archive_table(
    pool: &PgPool,
) -> (i32,i32) {
    let latest_edit_note = get_latest_edit_note_id(pool).await;
    let latest_edit = get_latest_edit_data_id(pool).await;
    println!("Latest edit: {}, note: {}", latest_edit, latest_edit_note);
    // 0th-> Edit Data, 1st -> Edit Note
    //TODO: Uncomment it later
    // (latest_edit, latest_edit_note)
    (111451813, 71025805)
}

pub async fn get_latest_edit_data_id(
    pool: &PgPool
) -> i32 {
    let select_latest_edit_data_row = "
         SELECT DISTINCT ON (edit)
         *
         FROM edit_data
         ORDER BY edit  DESC limit 1";

    let latest_edit_data_row = sqlx::query_as::<_, EditData>(
        select_latest_edit_data_row
    ).fetch_one(pool)
        .await;
    latest_edit_data_row.unwrap().edit
}

pub async fn get_latest_edit_note_id(
    pool: &PgPool
) -> i32 {
    let select_latest_edit_note_row = "
         SELECT DISTINCT ON (id)
         *
         FROM edit_note
         ORDER BY id  DESC LIMIT 1";

    let latest_edit_note_row = sqlx::query_as::<_, EditNote>(
        select_latest_edit_note_row
    ).fetch_one(pool)
        .await;
    latest_edit_note_row.unwrap().id
}

/// This function takes input a URL string, and returns true if it should exclude the URL from saving
pub fn should_exclude_url(url: &str) -> bool {
    // TODO: discuss and add keywords to identify URLs we want to exclude
    let keywords: Vec<&str> = vec!["www.musicbrainz.org", "https://musicbrainz.org", "www.metabrainz.org", "https://metabrainz.org", "web.archive.org"];
    keywords.iter().any(|keyword| url.contains(keyword))
}

///This function checks if we are inserting the same url within a day into the internet_archive_urls table
pub async fn should_insert_url_to_internet_archive_urls(
    url: &str,
    pool: &PgPool
) -> Result<bool, Error> {
    if should_exclude_url(url) {
        return Ok(false)
    }
    let res: Option<(bool, )> = sqlx::query_as(
        r#"
        SELECT (CURRENT_TIMESTAMP - created_at) > INTERVAL '1 DAY' AS daydiff
        FROM external_url_archiver.internet_archive_urls
        WHERE url = $1
        "#)
        .bind(url)
        .fetch_optional(pool)
        .await?;
    if res.is_some() {
        let (bool_val,) = res.unwrap();
        return Ok(bool_val);
    } else {
        Ok(true)
    }
}

pub async fn save_url_to_internet_archive_urls(
    url: &str,
    from_table: &str,
    from_table_id: i32,
    pool: &PgPool) {
    match should_insert_url_to_internet_archive_urls(url, pool).await {
        Ok(can_insert) => {
            if can_insert {
                let query = r#"
                            INSERT INTO external_url_archiver.internet_archive_urls (url, from_table, from_table_id, retry_count, is_saved)
                            VALUES ($1, $2, $3, 0, false)"#;
                sqlx::query(query)
                    .bind(url)
                    .bind(from_table)
                    .bind(from_table_id)
                    .execute(pool)
                    .await
                    .unwrap();
            }
        }
        Err(e) => {
            eprintln!("Error saving {} into internet_archive_urls: {}", url, e)
        }
    }
}

#[cfg(test)]
#[path= "./tests/utils.rs"]
mod tests;