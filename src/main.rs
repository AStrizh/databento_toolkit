use crate::commands::download::download_history;
use crate::commands::get_quote::get_quote;
use clap::Parser;
use anyhow::Result;
use time::{Date, Month};
use crate::downloader::decode::{decode_all_in_dir};

mod client;
mod commands;
mod downloader;
mod types;


#[derive(Parser)]
#[command(name = "DatabentoToolkit")]
#[command(about = "Run tasks related to historical market data", long_about = None)]
struct Args {
    /// Task to run: download, quote, or decode
    #[arg(value_enum, default_value_t = Task::Download)]
    task: Task,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Task {
    Download,
    Quote,
    Decode,
}



#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let base_path = "Hist_Fut_Data";

    match args.task {
        Task::Download => download_history(
            Date::from_calendar_date(2022, Month::February, 1).unwrap(),
            Date::from_calendar_date(2024, Month::May, 31).unwrap(),
            &["CL","ES"],
            base_path).await?,
        Task::Quote => get_quote().await?,
        Task::Decode => decode_all_in_dir(base_path).await?,
    }

    Ok(())
}