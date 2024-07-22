use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
pub struct ArchivalSuccessResponse {
    pub url: String,
    pub job_id: String,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ArchivalErrorResponse {
    pub message: String,
    pub status: String,
    pub status_ext: String,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ArchivalHtmlResponse {
    pub html: String,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum ArchivalResponse {
    Ok(ArchivalSuccessResponse),
    Err(ArchivalErrorResponse),
    Html(ArchivalHtmlResponse),
}
