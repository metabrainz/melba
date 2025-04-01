use core::num::NonZeroU32;
use std::sync::LazyLock;

use governor::clock::QuantaClock;
use governor::clock::QuantaInstant;
use governor::middleware::NoOpMiddleware;
use governor::state::InMemoryState;
use governor::state::NotKeyed;
use governor::Quota;
use governor::RateLimiter;

use crate::archival::archival_response::ArchivalErrorResponse;
use crate::archival::archival_response::ArchivalResponse;
use crate::archival::archival_response::ArchivalStatusResponse;
use crate::archival::client::REQWEST_CLIENT;
use crate::archival::error::ArchivalError;
use crate::configuration::SETTINGS;

pub static IA_SAVE_RATELIMIT: LazyLock<
    RateLimiter<NotKeyed, InMemoryState, QuantaClock, NoOpMiddleware<QuantaInstant>>,
> = LazyLock::new(|| {
    RateLimiter::direct(
        Quota::per_minute(NonZeroU32::new(SETTINGS.wayback_machine_api.save_rate_limit).unwrap())
            .allow_burst(NonZeroU32::new(1).unwrap()),
    )
});

pub static IA_STATUS_RATELIMIT: LazyLock<
    RateLimiter<NotKeyed, InMemoryState, QuantaClock, NoOpMiddleware<QuantaInstant>>,
> = LazyLock::new(|| {
    RateLimiter::direct(
        Quota::per_minute(NonZeroU32::new(SETTINGS.wayback_machine_api.status_rate_limit).unwrap())
            .allow_burst(NonZeroU32::new(1).unwrap()),
    )
});

/// Handles the network request to archive the URL
pub async fn archive_url_in_ia(url: &str) -> Result<ArchivalResponse, ArchivalError> {
    IA_SAVE_RATELIMIT.until_ready().await;

    let response = REQWEST_CLIENT
        .post(&SETTINGS.wayback_machine_api.save_endpoint_url)
        .body(format!("url={}", url))
        .send()
        .await?;

    let res = response.text().await?;

    // IA might return either a Ok response, or a JSON response, and occasionally a html response.
    // So we sort those in their proper types
    if let Ok(val) = serde_json::from_str(&res) {
        Ok(val)
    } else if let Ok(err) = serde_json::from_str::<ArchivalErrorResponse>(&res) {
        Err(ArchivalError::WaybackMachineErr(err))
    } else {
        Err(ArchivalError::WaybackMachineErrStr(res))
    }
}

// Handles the network request to get the status of the archival
pub async fn archiving_job_status(job_id: &str) -> Result<ArchivalStatusResponse, ArchivalError> {
    IA_STATUS_RATELIMIT.until_ready().await;

    let response = REQWEST_CLIENT
        .post(&SETTINGS.wayback_machine_api.status_endpoint_url)
        .body(format!("job_id={}", job_id))
        .send()
        .await?;

    let response_text = response.text().await?;

    ArchivalStatusResponse::from_body(response_text)
}
