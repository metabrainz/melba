use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

mod app;
mod archival;
mod cli;
mod poller;
mod structs;

mod configuration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let hostname = env::var("PGHOST").expect("PGHOST env variable is not set");
    //TODO: Check how to use Config with docker, and add Database config in config files
    let db_url = format!(
        "postgres://musicbrainz:musicbrainz@{}:5432/musicbrainz_db",
        hostname
    );
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    cli::start(&pool).await;
    Ok(())
}
