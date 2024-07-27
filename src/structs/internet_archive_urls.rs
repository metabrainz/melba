// Generated with sql-gen
// https://github.com/jayy-lmao/sql-gen
use crate::structs::internet_archive_urls::ArchivalStatus::{
    Failed, NotStarted, Processing, StatusError, Success,
};
use serde::Deserialize;
use sqlx::types::chrono;

#[derive(sqlx::Type, Debug, Clone, PartialEq)]
#[repr(i32)]
pub enum ArchivalStatus {
    NotStarted = 1,
    Processing = 2,
    Success = 3,
    Failed = 4,
    StatusError = 5,
}

impl TryFrom<i32> for ArchivalStatus {
    type Error = ();
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 if 1 == NotStarted as i32 => Ok(NotStarted),
            2 if 2 == Processing as i32 => Ok(Processing),
            3 if 3 == Success as i32 => Ok(Success),
            4 if 4 == Failed as i32 => Ok(Failed),
            5 if 5 == StatusError as i32 => Ok(StatusError),
            _ => Err(()),
        }
    }
}

#[derive(sqlx::FromRow, Debug, Deserialize, Clone)]
pub struct InternetArchiveUrls {
    pub id: i32,
    pub url: Option<String>,
    pub job_id: Option<String>,
    pub from_table: Option<String>,
    pub from_table_id: Option<i32>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub retry_count: Option<i32>,
    pub status: i32,
    pub status_message: Option<String>,
}
