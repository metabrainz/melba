use crate::cli::args::{CliArgs, Commands};
use crate::cli::utils::check_before_inserting_url;
use clap::Parser;
use colorize::AnsiColor;
use sqlx::PgPool;

mod args;
mod utils;

pub async fn start(pool: &PgPool) {
    let args = CliArgs::parse();
    match &args.command {
        Some(Commands::QueueEditData { row_id }) => {
            match utils::insert_edit_data_row_to_internet_archive_urls(*row_id, pool).await {
                // We got urls!
                Ok(true) => println!(
                    "{} {}",
                    "EditData's row enqueued successfully".green(),
                    row_id
                ),

                // We don't have urls
                Ok(false) => println!("{}", "No URLs found".red()),

                // Error
                Err(err) => {
                    println!("{}", "Some error occurred".red());
                    eprintln!("{err}");
                }
            }
        }

        Some(Commands::QueueEditNote { row_id }) => {
            match utils::insert_edit_note_row_to_internet_archive_urls(*row_id, pool).await {
                // We got urls!
                Ok(true) => println!(
                    "{} {}",
                    "EditNote's row enqueued successfully".green(),
                    row_id
                ),

                // We don't have urls
                Ok(false) => println!("{}", "No URLs found".red()),

                // Error
                Err(err) => {
                    println!("{}", "Some error occurred".red());
                    eprintln!("{err}");
                }
            }
        }
        Some(Commands::QueueURL { url }) => match check_before_inserting_url(url, pool).await {
            Ok(false) => {
                println!("{}", "URL is already queued: ".red(),);
            }
            Err(err) => {
                println!("{}", "Some error occurred".red());
                eprintln!("{err}");
            }

            Ok(true) => match utils::insert_url_to_internet_archive_urls(url, pool).await {
                Ok(id) => println!(
                    "{} {}",
                    "URL queued in internet_archive_urls, id: ".green(),
                    id
                ),

                Err(err) => {
                    println!("{}", "Some error occurred".red());
                    eprintln!("{err}");
                }
            },
        },
        Some(Commands::CheckStatus { job_id }) => {
            match utils::get_job_id_status(job_id.as_str(), pool).await {
                Ok(res) => {
                    println!("Status: {}", res.status);
                    println!("{:?}", res)
                }
                Err(e) => {
                    println!("Failed: {}", e)
                }
            }
        }

        Some(Commands::Poll) | None => {
            super::app::start(pool).await.unwrap();
        }
    }
}
