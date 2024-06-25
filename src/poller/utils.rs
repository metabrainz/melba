use linkify::{LinkFinder, LinkKind};
use serde_json::json;
use sqlx::{Error, PgPool};
use sqlx::types::JsonValue;
use crate::structs::internet_archive_urls::InternetArchiveUrls;

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

/// This function takes input a URL string, and returns true if it should exclude the URL from saving
pub fn should_exclude_url(url: &str) -> bool {
    // TODO: discuss and add keywords to identify URLs we want to exclude
    let keywords: Vec<&str> = vec!["musicbrainz", "metabrainz"];
    keywords.iter().any(|keyword| url.contains(keyword))
}

/// This function takes input Edit Data in form of JSONValue, checks if the Edit Data contains URL, and returns the URL as String
pub fn extract_url_from_edit_data(json: JsonValue) -> Vec<String> {
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

//TODO: Handle: 1. Can we/should we retrieve latest rows faster?
///This function fetches the latest row from internet_archive_urls_table
pub async fn extract_last_rows_idx_from_internet_archive_table(
    pool: &PgPool
) -> Vec<i32> {
    let last_row = sqlx::query_as::<_, InternetArchiveUrls>(
        "
        SELECT DISTINCT ON (from_table)
        id, url, job_id, from_table, from_table_id, created_at, retry_count, is_saved
        FROM external_url_archiver.internet_archive_urls
        WHERE from_table IN ('edit_data', 'edit_note')
        ORDER BY from_table, from_table_id DESC;
        "
    )
        .fetch_all(pool)
        .await;
    return match last_row {
        Ok(res) => {
            if res.is_empty(){
                return initialise_internet_archive_table(pool).await
            }
            return vec![res[0].from_table_id.unwrap(), res[1].from_table_id.unwrap()]
        },
        Err(_e) => initialise_internet_archive_table(pool).await
    }
}

//TODO: Make the following logic better!
///This function should run when there is no internet_archive_urls table or the table is not populated
pub async fn initialise_internet_archive_table(
    pool: &PgPool,
) -> Vec<i32> {
    create_internet_archive_urls_table(pool).await;
    //TODO: uncomment it later and replace the hardcoded ids with fetched ones,
    // and also insert them to internet_archive_urls table

    // let  select_latest_edit_data_row = "
    //      SELECT DISTINCT ON (edit)
    //      *
    //      FROM edit_data
    //      ORDER BY edit  DESC limit 1";
    //
    // let latest_edit_data_row = sqlx::query_as::<_,EditData>(select_latest_edit_data_row)
    //     .fetch_one(pool)
    //     .await;
    //
    // let select_latest_edit_note_row = "
    //      SELECT DISTINCT ON (id)
    //      *
    //      FROM edit_note
    //      ORDER BY id  DESC limit 1";
    //
    // let latest_edit_note_row = sqlx::query_as::<_, EditNote>(select_latest_edit_note_row)
    //     .fetch_one(pool)
    //     .await;
    // let latest_edit_note = latest_edit_note_row.unwrap().id;
    // let latest_edit = latest_edit_data_row.unwrap().edit;
    // println!("{}, note: {}", latest_edit, latest_edit_note);
    //0th-> Edit Data, 1st -> Edit Note
    return vec![48470658, 70000000]
}

///Initiate internet_archive_urls table
/// For development, adding 2 rows initially for the sake of demonstration TODO: Remove insert statements
async fn create_internet_archive_urls_table(
    pool: &PgPool
) {

    let sample_edit_note_row = "INSERT INTO external_url_archiver.internet_archive_urls
    (url, from_table, from_table_id, retry_count, is_saved) VALUES
    ('https://blackpaintingsdiscography.bandcamp.com/album/asmodea', 'edit_note', 70000000, 0, false);";

    let sample_edit_data_row = "INSERT INTO external_url_archiver.internet_archive_urls
    (url, from_table, from_table_id, retry_count, is_saved) VALUES
    ('http://rut-hc.bandcamp.com/album/demo', 'edit_data', 48470658 , 0, false);";

    sqlx::query(sample_edit_data_row)
        .execute(pool)
        .await
        .unwrap();

    sqlx::query(sample_edit_note_row)
        .execute(pool)
        .await
        .unwrap();
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
        let bool_val = res.unwrap().0;
        return Ok(bool_val);
    } else {
        Ok(true)
    }
}

#[cfg(test)]
#[path= "./tests/utils.rs"]
mod tests;