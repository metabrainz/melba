//! All the relevant models from the Msuicbrainz Database

/// The edit note of an entity
#[derive(sqlx::FromRow, Debug)]
pub struct EditNote {
    pub id: i32,
    pub editor: i32,
    pub edit: i32,
    pub text: String,

    #[allow(dead_code)]
    pub post_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// The data of the edited entity
#[derive(sqlx::FromRow, Debug)]
pub struct EditData {
    pub edit: i32,
    pub data: serde_json::Value,
}
