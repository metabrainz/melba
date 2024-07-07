use crate::cli::args::{CliArgs, Commands};
use clap::Parser;
use colorize::AnsiColor;
use sqlx::PgPool;

mod args;
mod utils;

pub async fn start(pool: &PgPool) {
    let args = CliArgs::parse();
    match &args.command {
        None => {
            super::app::start(pool).await.unwrap();
        }
        Some(Commands::QueueEditData { row_id }) => {
            if row_id.is_none() {
                println!("{}", "Please pass row id".red());
                return;
            }
            if let Ok(contains_urls) =
                utils::insert_edit_data_row_to_internet_archive_urls(row_id.unwrap(), pool).await
            {
                if contains_urls {
                    println!(
                        "{} {}",
                        "EditData's row enqueued successfully".green(),
                        row_id.unwrap()
                    )
                } else {
                    println!("{}", "No URLs found".red())
                }
            } else {
                println!("{}", "Some error occurred".red())
            }
        }
        Some(Commands::QueueEditNote { row_id }) => {
            if row_id.is_none() {
                println!("{}", "Please pass row id".red());
                return;
            }
            if let Ok(contains_urls) =
                utils::insert_edit_note_row_to_internet_archive_urls(row_id.unwrap(), pool).await
            {
                if contains_urls {
                    println!(
                        "{} {}",
                        "EditNote's row enqueued successfully".green(),
                        row_id.unwrap()
                    )
                } else {
                    println!("{}", "No URLs found".red())
                }
            } else {
                println!("{}", "Some error occurred".red())
            }
        }
        Some(Commands::QueueURL { url }) => {
            if url.is_none() {
                println!("{}", "Please pass URL".red());
                return;
            }
            if let Ok(id) =
                utils::insert_url_to_internet_archive_urls(url.clone().unwrap(), pool).await
            {
                println!(
                    "{} {}",
                    "URL queued in internet_archive_urls, id: ".green(),
                    id
                )
            } else {
                println!("{}", "Some error occurred".red())
            }
        }
        Some(Commands::CheckStatus { job_id }) => {
            if job_id.is_none() {
                println!("{}", "Please pass job id".red());
                return;
            }
            println!("{:?}", job_id);
            utils::get_job_id_status(job_id.clone().unwrap(), pool)
                .await
                .unwrap();
        }
        Some(Commands::Poll) => {
            super::app::start(pool).await.unwrap();
        }
    }
}
