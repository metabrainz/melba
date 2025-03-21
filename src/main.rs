use crate::configuration::SETTINGS;
use log::debug;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

mod app;
mod archival;
mod cli;
mod models;
mod poller;
mod structs;
mod models;

mod configuration;
mod metrics;

fn main() {
    SETTINGS.init_logger();
    let _guard = if !SETTINGS.sentry.url.trim().is_empty() {
        debug!("Initializing Sentry with DSN...");
        Some(sentry::init((
            SETTINGS.sentry.url.as_str(),
            sentry::ClientOptions {
                release: sentry::release_name!(),
                ..Default::default()
            },
        )))
    } else {
        debug!("Sentry DSN is not provided, skipping Sentry initialization.");
        None
    };

    // Initialize the Tokio runtime
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let hostname = &SETTINGS.database.pg_host;
            let user = &SETTINGS.database.pg_user;
            let password = &SETTINGS.database.pg_password;
            let port = SETTINGS.database.pg_port;
            let db = &SETTINGS.database.pg_database;

            let connect_options = PgConnectOptions::new()
                .host(hostname)
                .port(port)
                .username(user)
                .password(password)
                .database(db)
                .statement_cache_capacity(0);

            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect_lazy_with(connect_options);

            cli::start(&pool).await;
        });
}
