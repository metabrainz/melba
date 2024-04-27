mod looper;
mod utils;

use std::time::Duration;
use sqlx::Row;
use tokio::time::interval;

pub struct Poller {
    poll_interval: u64,
    pool: sqlx::PgPool
}

impl Poller {
    pub fn new(
        poll_interval: u64,
        pool: sqlx::PgPool) -> Poller {
        Poller {
            poll_interval,
            pool
        }
    }
    pub async fn poll(&self) {
        let mut interval = interval(Duration::from_secs(self.poll_interval));
        loop {
            interval.tick().await;
            if let Err(e) = looper::poll_db(&self.pool).await {
                eprintln!("Error polling database: {}", e)
            }
        }
    }
}