use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
pub struct ArchivalResponse {
    pub url: String,
    pub job_id: String,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ArchivalErrorResponse {
    pub message: String,
    pub status: String,
    pub status_ext: String,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct ArchivalStatusResponse {
    pub duration_sec: Option<f64>,
    pub http_status: Option<i64>,
    pub job_id: String,
    pub original_url: Option<String>,
    pub status: String,
    pub timestamp: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
pub struct ArchivalStatusErrorResponse {
    pub job_id: String,
    pub message: String,
    pub status_ext: String,
    pub status: Option<String>,
}
