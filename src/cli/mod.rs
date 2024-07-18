use crate::cli::args::{CliArgs, Commands};
use clap::Parser;
use colorize::AnsiColor;
use sqlx::PgPool;

mod args;
mod utils;

pub async fn start(pool: &PgPool) {
    let args = CliArgs::parse();
    match &args.command {
        Some(Commands::QueueEditData { row_id }) => {
            // Argument check
            // TODO: Concider making the argument mandatory by removing the `Option`?
            let Some(row_id) = row_id else {
                println!("{}", "Please pass row id".red());
                return;
            };

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
            // Argument check
            // TODO: Concider making the argument mandatory by removing the `Option`?
            let Some(row_id) = row_id else {
                println!("{}", "Please pass row id".red());
                return;
            };

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
        Some(Commands::QueueURL { url }) => {
            // Argument check
            // TODO: Concider making the argument mandatory by removing the `Option`?
            let Some(url) = url else {
                println!("{}", "Please pass URL".red());
                return;
            };

            match utils::insert_url_to_internet_archive_urls(url, pool).await {
                Ok(id) => println!(
                    "{} {}",
                    "URL queued in internet_archive_urls, id: ".green(),
                    id
                ),

                Err(err) => {
                    println!("{}", "Some error occurred".red());
                    eprintln!("{err}");
                }
            }
        }
        Some(Commands::CheckStatus { job_id }) => {
            // Argument check
            // TODO: Concider making the argument mandatory by removing the `Option`?
            let Some(job_id) = job_id else {
                println!("{}", "Please pass job id".red());
                return;
            };

            println!("Job id: {:?}", job_id);
            utils::get_job_id_status(job_id.to_owned(), pool)
                .await
                .unwrap();
        }

        Some(Commands::Poll) | None => {
            super::app::start(pool).await.unwrap();
        }
    }
}
