pub mod edit_types;
pub mod looper;
pub mod utils;

use crate::poller::utils::{get_edit_data_and_note_start_id, update_last_unprocessed_rows};
use sqlx::Error;
use std::time::Duration;
use tokio::time::interval;

/// Responsible for polling musicbrainz data for edit_notes and edit_data
pub struct Poller {
    /// Time between polls, in seconds
    poll_interval: u64,

    /// The database pool
    pool: sqlx::PgPool,

    /// The start id of the note poller
    edit_note_start_idx: i32,

    /// The start id of the data poller
    edit_data_start_idx: i32,
}

impl Poller {
    pub async fn new(poll_interval: u64, pool: sqlx::PgPool) -> Result<Self, Error> {
        // Fetch the start ids for the poller from the DB
        let (edit_data_start_idx, edit_note_start_idx) =
            get_edit_data_and_note_start_id(&pool).await?;

        Ok(Poller {
            poll_interval,
            pool,
            edit_data_start_idx,
            edit_note_start_idx,
        })
    }

    /// This function is the main loop of the poller. Each loop polls the `edit_data` and `edit_note` tables for new changes
    pub async fn run(&mut self) -> Result<(), Error> {
        let mut interval = interval(Duration::from_secs(self.poll_interval));

        loop {
            interval.tick().await;

            self.poll().await?;
        }
    }

    /// Execute a poll action.
    async fn poll(&mut self) -> Result<(), Error> {
        let (edit_data_id, edit_note_id) = looper::poll_db(
            &self.pool,
            self.edit_data_start_idx,
            self.edit_note_start_idx,
        )
        .await?;

        if let Some(edit_data_id) = edit_data_id {
            self.edit_data_start_idx = edit_data_id;
            update_last_unprocessed_rows("edit_data", edit_data_id, &self.pool).await?;
        }

        if let Some(edit_note_id) = edit_note_id {
            self.edit_note_start_idx = edit_note_id;
            update_last_unprocessed_rows("edit_note", edit_note_id, &self.pool).await?;
        }

        Ok(())
    }
}
