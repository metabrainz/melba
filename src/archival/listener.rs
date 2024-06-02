use sqlx::{Error, PgPool};
use sqlx::postgres::PgListener;

pub async fn listen(pool: PgPool) -> Result<(), Error> {
    println!("Listener Task");
    let mut listener = PgListener::connect_with(&pool).await?;
    listener.listen("archive_urls").await.unwrap();
    loop {
        let notification = listener.recv().await.unwrap();
        println!("{}", notification.payload());
        //TODO:archive here
    }
}