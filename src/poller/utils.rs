use regex::Regex;

/// This function takes text from edit note and outputs a vector of URLs as string
pub fn extract_urls_from_edit_note(note: &str) -> Vec<String> {
    //TODO: test regex on more URLs
    //Used regex mentioned here: https://stackoverflow.com/a/29288898/15084244
    let re = Regex::new(r"(?:(?:https?|ftp|file):\/\/|www\.|ftp\.)(?:\([-A-Z0-9+&@#\/%=~_|$?!:,.]*\)|[-A-Z0-9+&@#\/%=~_|$?!:,.])*(?:\([-A-Z0-9+&@#\/%=~_|$?!:,.]*\)|[A-Z0-9+&@#\/%=~_|$])").unwrap();
    re.find_iter(note)
        .map(|mat| mat.as_str().to_string())
        .collect()
}

/// This function takes input a URL string, and returns true if it should exclude the URL from saving
pub fn should_exclude_url(url: &str) -> bool {
    // TODO: discuss and add keywords to identify URLs we want to exclude
    let keywords: Vec<String> = vec![];
    keywords.iter().any(|keyword| url.contains(keyword))
}