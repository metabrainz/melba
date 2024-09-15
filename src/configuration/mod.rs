use config::{Config, ConfigError, File};
use dotenv::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct WaybackMachineApi {
    pub myaccesskey: String,
    pub mysecret: String,
}

#[derive(Debug, Deserialize)]
pub struct RetryTask {
    pub select_limit: i32,
    pub retry_interval: u64,
    pub allow_remove_row_after: i64,
}

#[derive(Debug, Deserialize)]
pub struct PollerTask {
    pub poll_interval: u64,
}

#[derive(Debug, Deserialize)]
pub struct NotifyTask {
    pub notify_interval: u64,
}

#[derive(Debug, Deserialize)]
pub struct ListenTask {
    pub listen_interval: u64,
    pub sleep_status_interval: u64,
}

#[derive(Debug, Deserialize)]
pub struct Sentry {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub pg_host: String,
    pub pg_port: u16,
    pub pg_user: String,
    pub pg_password: String,
    pub pg_database: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub wayback_machine_api: WaybackMachineApi,
    pub retry_task: RetryTask,
    pub poller_task: PollerTask,
    pub notify_task: NotifyTask,
    pub listen_task: ListenTask,
    pub sentry: Sentry,
    pub database: Database,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        dotenv().ok();
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        let config = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            .build()?;
        config.try_deserialize()
    }
}
