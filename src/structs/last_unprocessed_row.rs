use serde::Deserialize;
use sqlx::FromRow;

#[derive(Debug, Deserialize, FromRow)]
pub struct LastUnprocessedRow {
    pub id_column: i32,
    #[allow(dead_code)]
    pub table_name: String,
}
