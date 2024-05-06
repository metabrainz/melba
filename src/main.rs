use sqlx::postgres::PgPoolOptions;
use crate::poller::Poller;

mod poller;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const POLL_INTERVAL: u64 = 10;
    //NOTE: for time being, keeping the db_url to a custom db, will check with mb schema later
    let db_url = "postgres://musicbrainz:musicbrainz@localhost:5432/musicbrainz_db";
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    let poller = Poller::new(POLL_INTERVAL, pool);
    tokio::spawn(async move {
        poller
            .poll()
            .await;
    }).await.unwrap();
    Ok(())
}
