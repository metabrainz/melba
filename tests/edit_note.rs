#[cfg(test)]
mod edit_note_tests {
    use mb_rs::schema::EditNote;
    use sqlx::{PgPool, query_as};

    #[sqlx::test(fixtures("EditNote"))]
    async fn test_select_edit_note(pool: PgPool) -> sqlx::Result<()> {
       let row = query_as::<_, EditNote>(
            r#"
                SELECT *
                FROM edit_note
                WHERE edit = 111451706
            "#
            ).fetch_one(&pool)
           .await.unwrap();
        assert_eq!(row.edit, 111451706);
        Ok(())
    }
}