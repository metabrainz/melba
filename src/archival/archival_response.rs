use serde::Deserialize;
use std::fmt;

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

#[derive(Default, Clone, PartialEq, Deserialize)]
pub struct ArchivalStatusResponse {
    pub duration_sec: Option<f64>,
    pub http_status: Option<i64>,
    pub job_id: String,
    pub original_url: Option<String>,
    pub status: String,
    pub timestamp: Option<String>,
}

#[derive(Default, Clone, PartialEq, Deserialize)]
pub struct ArchivalStatusErrorResponse {
    pub job_id: String,
    pub message: Option<String>,
    pub status_ext: Option<String>,
    pub status: Option<String>,
}

impl fmt::Debug for ArchivalStatusResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Archival Status Response")
            .field("duration_sec", &self.duration_sec.unwrap_or_default())
            .field("http_status", &self.http_status.unwrap_or_default())
            .field("job_id", &self.job_id)
            .field("original_url", &self.original_url.as_deref().unwrap_or(""))
            .field("status", &self.status)
            .field("timestamp", &self.timestamp.as_deref().unwrap_or(""))
            .finish()
    }
}

impl fmt::Debug for ArchivalStatusErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Archival Status Error Response")
            .field("job_id", &self.job_id)
            .field("message", &self.message.as_deref().unwrap_or("")) // Unwrap Option<String> or use an empty string
            .field("status_ext", &self.status_ext.as_deref().unwrap_or("")) // Unwrap Option<String> or use an empty string
            .field("status", &self.status.as_deref().unwrap_or("")) // Unwrap Option<String> or use an empty string
            .finish()
    }
}
