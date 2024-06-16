use linkify::{LinkFinder, LinkKind};
use sqlx::{Error, PgPool};
use crate::structs::internet_archive_urls::InternetArchiveUrls;

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

//TODO: Handle: 1. Can we/should we retrieve latest rows faster?
///This function fetches the latest row from internet_archive_urls_table
pub async fn extract_last_rows_idx_from_internet_archive_table(
    pool: &PgPool
) -> i32 {
    let last_row = sqlx::query_as::<_, InternetArchiveUrls>(
        "
        SELECT DISTINCT ON (from_table)
        id, url, job_id, from_table, from_table_id, created_at, retry_count, is_saved
        FROM external_url_archiver.internet_archive_urls
        WHERE from_table = 'edit_note'
        ORDER BY from_table, from_table_id DESC;
        "
    )
        .fetch_one(pool)
        .await;
    return match last_row {
        Ok(res) => {
            return res.from_table_id.unwrap();
        },
        Err(_e) => initialise_internet_archive_table(pool).await
    }
}

//TODO: Make the following logic better!
///This function should run when there is no internet_archive_urls table or the table is not populated
pub async fn initialise_internet_archive_table(
    pool: &PgPool,
) -> i32 {
    create_internet_archive_urls_table(pool).await;
    //TODO: uncomment it later and replace the hardcoded ids with fetched ones, and also insert them to internet_archive_urls table

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
    // println!("note: {}", latest_edit_note);

    return 70000000;
}

///Initiate internet_archive_urls table
/// For development, adding 2 rows initially for the sake of demonstration TODO: Remove insert statements
async fn create_internet_archive_urls_table(
    pool: &PgPool
) {

    let sample_edit_note_row = "INSERT INTO external_url_archiver.internet_archive_urls
    (url, from_table, from_table_id, retry_count, is_saved) VALUES
    ('https://blackpaintingsdiscography.bandcamp.com/album/asmodea', 'edit_note', 70000000, 0, false);";

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