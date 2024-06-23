use std::env;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;

mod poller;
mod archival;
mod structs;
mod cli;
mod app;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    dotenv().ok();

    let hostname = env::var("PGHOST").expect("PGHOST env variable is not set");

    //TODO: How to manage prod DB and dev DB?
    let db_url = format!("postgres://musicbrainz:musicbrainz@{}:5432/musicbrainz_db", hostname);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    cli::start(&pool).await;
    Ok(())
}