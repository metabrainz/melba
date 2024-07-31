use crate::archival::archival_response::{ArchivalErrorResponse, ArchivalStatusErrorResponse};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArchivalError {
    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Could not deserialize, error: {0}")]
    JsonDeserialization(#[from] serde_json::Error),

    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("archival error: {0:?}")]
    SaveRequestError(ArchivalErrorResponse),

    #[error("archival status error {0:?}")]
    StatusRequestErrorResponse(ArchivalStatusErrorResponse),

    #[error("HTML Response: {0}")]
    HtmlResponse(String),
}
