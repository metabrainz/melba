pub mod looper;
pub mod utils;

use crate::poller::utils::{get_edit_data_and_note_start_id, update_last_unprocessed_rows};
use sqlx::Error;
use std::time::Duration;
use tokio::time::interval;

/// Responsible for polling musicbrainz data for edit_notes and edit_data
pub struct Poller {
    poll_interval: u64,
    pool: sqlx::PgPool,
    edit_note_start_idx: i32,
    edit_data_start_idx: i32,
}

impl Poller {
    pub async fn new(poll_interval: u64, pool: sqlx::PgPool) -> Result<Self, Error> {
        let (edit_data_start_idx, edit_note_start_idx) =
            get_edit_data_and_note_start_id(&pool).await?;
        Ok(Poller {
            poll_interval,
            pool,
            edit_data_start_idx,
            edit_note_start_idx,
        })
    }

    /// Polls the `edit_data` and `edit_note` tables continuously
    pub async fn poll(&mut self) -> Result<(), Error> {
        let mut interval = interval(Duration::from_secs(self.poll_interval));
        loop {
            interval.tick().await;
            match looper::poll_db(
                &self.pool,
                self.edit_data_start_idx,
                self.edit_note_start_idx,
            )
            .await
            {
                Ok((edit_data_id, edit_note_id)) => {
                    if edit_data_id.is_some() {
                        self.edit_data_start_idx = edit_data_id.unwrap();
                        update_last_unprocessed_rows(
                            "edit_data",
                            edit_data_id.unwrap(),
                            &self.pool,
                        )
                        .await?;
                    }
                    if edit_note_id.is_some() {
                        self.edit_note_start_idx = edit_note_id.unwrap();
                        update_last_unprocessed_rows(
                            "edit_note",
                            edit_note_id.unwrap(),
                            &self.pool,
                        )
                        .await?;
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}
