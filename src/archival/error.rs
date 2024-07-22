use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArchivalError {
    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Could not deserialize error: {0}")]
    JsonDeserialization(#[from] serde_json::Error),

    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),
}
