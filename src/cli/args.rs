use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author="yellowhatpro", version="1", about="Archive MusicBrainz edit URLs to Internet Archive URLs", long_about=None)]
pub struct CliArgs {
    /// Choose between queueing a
    /// * URL to archive
    /// * row from EditNote to archive
    /// * row from EditData to archive
    ///
    /// or
    ///
    /// Get the status of a URL archival job
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    ///Queue a single URL to be archived in Internet Archive History
    QueueURL { url: String },
    /// Queue an Edit Data row to be archived in Internet Archive History
    QueueEditData { row_id: i32 },
    /// Queue an Edit Note row to be archived in Internet Archive History
    QueueEditNote { row_id: i32 },
    /// Check the archival status of any URL by `job_id`
    CheckStatus { job_id: String },
    /// Start the app to poll from Edit Data and Edit Note tables. It is the default behaviour
    Poll,
}
