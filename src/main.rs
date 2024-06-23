mod poller;
mod archival;
mod structs;
mod cli;
mod app;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::start().await;
    Ok(())
}