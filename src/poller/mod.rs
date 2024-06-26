mod looper;
pub mod utils;

use std::time::Duration;
use tokio::time::interval;
use crate::poller::utils::extract_last_rows_idx_from_internet_archive_table;

/// Responsible for polling musicbrainz data for edit_notes (edit_data is being handled by postgres trigger)
pub struct Poller {
    poll_interval: u64,
    pool: sqlx::PgPool,
    edit_note_start_idx: i32,
    edit_data_start_idx: i32
}

impl Poller {
    pub async fn new(
        poll_interval: u64,
        pool: sqlx::PgPool) -> Poller {
        let data_and_note = extract_last_rows_idx_from_internet_archive_table(&pool).await;
        Poller {
            poll_interval,
            pool,
            edit_data_start_idx: data_and_note[0],
            edit_note_start_idx: data_and_note[1]
        }
    }
    pub async fn poll(&mut self) {
        let mut interval = interval(Duration::from_secs(self.poll_interval));
        loop {
            interval.tick().await;
            if let Ok((edit_data_id, edit_note_id)) = looper::poll_db(&self.pool, self.edit_data_start_idx, self.edit_note_start_idx)
                .await {
                    if edit_data_id.is_some() {
                        self.edit_data_start_idx = edit_data_id.unwrap() + 1;
                    }
                    if edit_note_id.is_some() {
                        self.edit_note_start_idx += edit_note_id.unwrap() + 1;
                    }
            } else {
                eprintln!("Problem polling")
            }
        }
    }
}