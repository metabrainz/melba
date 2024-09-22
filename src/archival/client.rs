use crate::configuration::SETTINGS;
use once_cell::sync::Lazy;
use reqwest::{header, Client};

pub static REQWEST_CLIENT: Lazy<Client> = Lazy::new(|| {
    let mut headers = header::HeaderMap::new();
    headers.insert("Accept", "application/json".parse().unwrap());
    headers.insert(
        "Authorization",
        format!(
            "LOW {}:{}",
            SETTINGS.wayback_machine_api.myaccesskey, SETTINGS.wayback_machine_api.mysecret
        )
        .parse()
        .unwrap(),
    );
    headers.insert(
        "Content-Type",
        "application/x-www-form-urlencoded".parse().unwrap(),
    );

    Client::builder()
        .default_headers(headers)
        .build()
        .expect("Failed to build HTTP client")
});
