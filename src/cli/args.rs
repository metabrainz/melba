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
    QueueURL {
        url: Option<String>
    },
    QueueEditData {
        row_id: Option<i32>
    },
    QueueEditNote {
        row_id: Option<i32>
    },
    CheckStatus {
        job_id: Option<String>
    },
    Poll
}