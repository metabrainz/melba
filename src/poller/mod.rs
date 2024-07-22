mod looper;
pub mod utils;

use crate::poller::utils::get_edit_data_and_note_start_id;
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

    /// The start if of the data poller
    edit_data_start_idx: i32,
}

impl Poller {
    pub async fn new(poll_interval: u64, pool: sqlx::PgPool) -> Poller {
        // Fetch the start ids for the poller from the DB
        let (edit_data_start_idx, edit_note_start_idx) =
            get_edit_data_and_note_start_id(&pool).await;

        Poller {
            poll_interval,
            pool,
            edit_data_start_idx,
            edit_note_start_idx,
        }
    }

    /// This function is the main loop of the poller. Each loop polls the `edit_data` and `edit_note` tables for new changes
    pub async fn run(&mut self) {
        let mut interval = interval(Duration::from_secs(self.poll_interval));
        loop {
            interval.tick().await;

            let poll_future = looper::poll_db(
                &self.pool,
                self.edit_data_start_idx,
                self.edit_note_start_idx,
            );

            match poll_future.await {
                Ok((edit_data_id, edit_note_id)) => {
                    // Set the next data's id
                    if let Some(edit_data_id) = edit_data_id {
                        self.edit_data_start_idx = edit_data_id;
                    }

                    // Set the next note's id
                    if let Some(edit_note_id) = edit_note_id {
                        self.edit_data_start_idx = edit_note_id;
                    }
                }
                Err(e) => {
                    eprintln!("Problem polling {}", e)
                }
            }
        }
    }
}
