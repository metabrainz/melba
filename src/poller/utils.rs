use linkify::{LinkFinder, LinkKind};
use serde_json::json;
use sqlx::types::JsonValue;

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