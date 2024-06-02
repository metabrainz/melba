use sqlx::PgPool;

pub async fn notify(pool: &PgPool){
    let res = sqlx::query(
        r#"
        DO $$
        DECLARE
            rec RECORD;
        BEGIN
            FOR rec IN SELECT * FROM internet_archive_urls
            LOOP
                PERFORM pg_notify('archive_urls', row_to_json(rec)::text);
            END LOOP;
        END $$;"#
    ).execute(pool)
        .await;
    println!("[from notify]: {res:?}");
}

