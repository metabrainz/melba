use clap::Parser;
use colorize::AnsiColor;
use sqlx::PgPool;
use crate::cli::args::{CliArgs, Commands};

mod args;
mod utils;

pub async fn start(pool: &PgPool) {
    let args = CliArgs::parse();
    match &args.command {
        None => {}
        Some(Commands::QueueEditData {
                 row_id
             }) => {
            if row_id.is_none() {
                println!("{}", "Please pass row id".red());
                return;
            }
            println!("{}", row_id.unwrap().to_string());
            utils::insert_edit_data_row_to_internet_archive_urls(row_id.unwrap(), &pool).await;
        },
        Some(Commands::QueueEditNote {
                 row_id
             }) => {
            if row_id.is_none() {
                println!("{}", "Please pass row id".red());
                return;
            }
            println!("{}", row_id.unwrap().to_string());
            utils::insert_edit_note_row_to_internet_archive_urls(row_id.unwrap(), &pool).await;
        },
        Some(Commands::QueueURL {
                 url
             }) => {
            if url.is_none() {
                println!("{}", "Please pass URL".red());
                return;
            }
            println!("{:?}", url);
            utils::insert_url_to_internet_archive_urls(url.clone().unwrap(), &pool).await;
        },
        Some(Commands::CheckStatus {
                 job_id
             }) => {
            if job_id.is_none() {
                println!("{}", "Please pass job id".red());
                return;
            }
            println!("{:?}", job_id);
            utils::get_job_id_status(job_id.clone().unwrap(), &pool).await.unwrap();
        },
        Some(Commands::Poll) => {
            super::app::start(&pool).await.unwrap();
        }
    }
}