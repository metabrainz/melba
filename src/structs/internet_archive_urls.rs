#![allow(dead_code)]
// Generated with sql-gen
// https://github.com/jayy-lmao/sql-gen
use sqlx::types::chrono;
use serde::{Deserialize};

#[derive(sqlx::FromRow, Debug, Deserialize)]
pub struct InternetArchiveUrls {
    pub id: i32,
    pub url: Option<String>,
    pub job_id: Option<String>,
    pub from_table: Option<String>,
    pub from_table_id: Option<i32>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub retry_count: Option<i32>,
    pub is_saved: Option<bool>,
}