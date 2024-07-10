use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ArchivalSuccessResponse {
    pub url: String,
    pub job_id: String,
}

#[derive(Deserialize, Debug)]
pub struct ArchivalErrorResponse {
    pub message: String,
    pub status: String,
    pub status_ext: String,
}