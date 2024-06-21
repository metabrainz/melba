#[cfg(test)]
mod edit_data_tests {
    use mb_rs::schema::EditData;
    use sqlx::{PgPool, query_as};

    #[sqlx::test(fixtures("EditData"))]
    async fn test_select_edit_data(pool: PgPool) -> sqlx::Result<()> {
        let row = query_as::<_, EditData>(
            r#"
                SELECT *
                FROM edit_data
                WHERE edit = 1
            "#
        ).fetch_one(&pool)
            .await.unwrap();
        assert_eq!(row.edit, 1);
        Ok(())
    }
}