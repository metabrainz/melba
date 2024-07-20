use config::{Config, ConfigError, File};
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
pub struct Settings {
    pub wayback_machine_api: WaybackMachineApi,
    pub retry_task: RetryTask,
    pub poller_task: PollerTask,
    pub notify_task: NotifyTask,
    pub listen_task: ListenTask,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        let config = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            .build()?;
        config.try_deserialize()
    }
}
