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
        let (edit_data_start_idx, edit_note_start_idx) = extract_last_rows_idx_from_internet_archive_table(&pool).await;
        Poller {
            poll_interval,
            pool,
            edit_data_start_idx,
            edit_note_start_idx
        }
    }

    /// Polls the `edit_data` and `edit_note` tables continuously
    pub async fn poll(&mut self) {
        let mut interval = interval(Duration::from_secs(self.poll_interval));
        loop {
            interval.tick().await;
            match looper::poll_db(&self.pool, self.edit_data_start_idx, self.edit_note_start_idx).await {
                Ok((edit_data_id, edit_note_id)) => {
                    if edit_data_id.is_some() {
                        self.edit_data_start_idx = edit_data_id.unwrap();
                    }
                    if edit_note_id.is_some() {
                        self.edit_note_start_idx = edit_note_id.unwrap();
                    }
                }
                Err(e) => {
                    eprintln!("Problem polling {}", e)
                }
            }
        }
    }
}