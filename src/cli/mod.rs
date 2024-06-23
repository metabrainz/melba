use clap::Parser;
use crate::cli::args::{CliArgs, Commands};

mod args;

pub async fn start() {
    let args = CliArgs::parse();
    match args.poll {
        None => {}
        Some(_) => {
            super::app::start().await.unwrap();
        }
    }
    match &args.command {
        None => {}
        Some(Commands::QueueEditData {
                 row_id
             }) => {
            println!("{}", row_id.unwrap().to_string())
        },
        Some(Commands::QueueEditNote {
                 row_id
             }) => {
            println!("{}", row_id.unwrap().to_string())
        },
        Some(Commands::QueueURL {
                 url
             }) => {
            println!("{:?}", url);
        },
        _ => {}
    }
}