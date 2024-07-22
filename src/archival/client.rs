use crate::configuration::Settings;
use once_cell::sync::Lazy;
use reqwest::{header, Client};
use std::sync::Arc;

pub static REQWEST_CLIENT: Lazy<Arc<Client>> = Lazy::new(|| {
    let settings = Settings::new().expect("Config needed");
    let mut headers = header::HeaderMap::new();
    headers.insert("Accept", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        format!(
            "LOW {}:{}",
            settings.wayback_machine_api.myaccesskey, settings.wayback_machine_api.mysecret
        )
        .parse()
        .unwrap(),
    );
    headers.insert(
        "Content-Type",
        "application/x-www-form-urlencoded".parse().unwrap(),
    );

    Arc::new(
        Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client"),
    )
});
