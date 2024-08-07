use crate::configuration::Settings;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

mod app;
mod archival;
mod cli;
mod poller;
mod structs;

mod configuration;
mod metrics;

fn main() {
    let settings = Settings::new().expect("Sentry Config not set");
    let _guard = sentry::init((
        settings.sentry.url,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));
    dotenv().ok();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
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
        });
}
