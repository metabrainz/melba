use crate::configuration::Settings;
use sqlx::postgres::PgPoolOptions;

mod app;
mod archival;
mod cli;
mod poller;
mod structs;

mod configuration;
mod metrics;

fn main() {
    let settings = Settings::new().expect("Failed to load settings");

    let _guard = if !settings.sentry.url.trim().is_empty() {
        println!("Initializing Sentry with DSN...");
        Some(sentry::init((
            settings.sentry.url.as_str(),
            sentry::ClientOptions {
                release: sentry::release_name!(),
                ..Default::default()
            },
        )))
    } else {
        println!("Sentry DSN is not provided, skipping Sentry initialization.");
        None
    };

    // Initialize the Tokio runtime
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let hostname = settings.database.pg_host;
            let user = settings.database.pg_user;
            let password = settings.database.pg_password;
            let port = settings.database.pg_port;
            let db = settings.database.pg_database;
            let db_url = format!(
                "postgres://{}:{}@{}:{}/{}",
                user, password, hostname, port, db
            );
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&db_url)
                .await
                .expect("Failed to connect to the database");

            cli::start(&pool).await;
        });
}
