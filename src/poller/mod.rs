mod looper;
mod utils;

use std::time::Duration;
use tokio::time::interval;
use crate::poller::utils::extract_last_row_idx_from_internet_archive_table;

/// Responsible for polling musicbrainz data for edit_notes (edit_data is being handled by postgres trigger)
pub struct Poller {
    poll_interval: u64,
    pool: sqlx::PgPool,
    edit_note_start_idx: i32,
}

impl Poller {
    pub async fn new(
        poll_interval: u64,
        pool: sqlx::PgPool) -> Poller {
        let edit_note_start_idx =  extract_last_row_idx_from_internet_archive_table(&pool).await;
        Poller {
            poll_interval,
            pool,
            edit_note_start_idx
        }
    }
    pub async fn poll(&mut self) {
        let mut interval = interval(Duration::from_secs(self.poll_interval));
        loop {
            interval.tick().await;
            if let Err(e) = looper::poll_db(&self.pool, self.edit_note_start_idx).await {
                eprintln!("Error polling database: {}", e)
            } else {
                // TODO: Check if incrementing edit note start index should depend on other aspects
                self.edit_note_start_idx = self.edit_note_start_idx + 10;
            }
        }
    }
}