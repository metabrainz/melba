use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct WaybackMachineApi {
    pub myaccesskey: String,
    pub mysecret: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub wayback_machine_api: WaybackMachineApi,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        let config = Config::builder()
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            .build()?;
        println!("debug: {:?}", config.get_bool("debug"));
        config.try_deserialize()
    }
}
