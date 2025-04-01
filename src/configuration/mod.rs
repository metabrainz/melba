use config::{Config, ConfigError, File};
use core::time::Duration;
use dotenv::dotenv;
use env_logger::Builder;
use log::LevelFilter;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::env;
use std::io::Write;

pub static SETTINGS: Lazy<Settings> =
    Lazy::new(|| Settings::new().expect("Failed to load settings"));

#[derive(Debug, Deserialize)]
pub struct WaybackMachineApi {
    pub myaccesskey: String,
    pub mysecret: String,
    pub save_endpoint_url: String,
    pub status_endpoint_url: String,
    pub save_rate_limit: u32,
    pub status_rate_limit: u32,
}

#[derive(Debug, Deserialize)]
pub struct PollerTask {
    pub poll_interval: u64,
}
#[derive(Debug, Deserialize)]
pub struct ArchivalTask {
    pub job_interval: u64,
    pub worker_count: u64,
    pub max_retry: i64,
    pub retry_interval: u64,
    pub allow_remove_row_after: i64,
}

#[derive(Debug, Deserialize)]
pub struct StatusWatchTask {
    pub job_interval: u64,
    pub worker_count: u64,
    pub max_retry: u64,
    pub retry_interval: u64,
}

#[derive(Debug, Deserialize)]
pub struct CleanupTask {
    pub job_interval: u64,
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

#[derive(Deserialize, Debug)]
pub struct Logs {
    pub debug: bool,
    pub error: bool,
    pub warning: bool,
    pub info: bool,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub wayback_machine_api: WaybackMachineApi,
    pub database: Database,

    // Task settings
    pub poller_task: PollerTask,
    pub archival_task: ArchivalTask,
    pub status_watch_task: StatusWatchTask,
    pub cleaner_task: CleanupTask,

    // Log settings
    pub sentry: Sentry,
    pub logs: Logs,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        dotenv().ok();
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        let config = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false));

        #[cfg(test)]
        let config = config.add_source(File::with_name("config/testing").required(false));

        let config = config.build()?;
        config.try_deserialize()
    }

    pub fn init_logger(&self) {
        let mut builder = Builder::new();
        if self.logs.error {
            builder.filter(None, LevelFilter::Error);
        }
        if self.logs.warning {
            builder.filter(None, LevelFilter::Warn);
        }
        if self.logs.info {
            builder.filter(None, LevelFilter::Info);
        }
        if self.logs.debug {
            builder.filter(None, LevelFilter::Debug);
        }
        if !self.logs.debug && !self.logs.info && !self.logs.warning && !self.logs.error {
            builder.filter(None, LevelFilter::Off);
        }

        builder.filter_module("sqlx", LevelFilter::Info);
        builder.filter_module("hyper", LevelFilter::Info);
        builder.filter_module("reqwest", LevelFilter::Info);
        builder.filter_module("h2", LevelFilter::Info);

        builder.format(|buf, record| writeln!(buf, "[{}] - {}", record.level(), record.args()));
        builder.init();
    }
}

impl ArchivalTask {
    pub fn get_job_interval(&self) -> Duration {
        Duration::new(self.job_interval, 0)
    }

    pub fn get_retry_interval(&self) -> Duration {
        Duration::new(SETTINGS.archival_task.retry_interval, 0)
    }
}

impl StatusWatchTask {
    pub fn get_job_interval(&self) -> Duration {
        Duration::new(self.job_interval, 0)
    }
}

impl CleanupTask {
    pub fn get_job_interval(&self) -> Duration {
        Duration::new(self.job_interval * 60, 0)
    }
}
