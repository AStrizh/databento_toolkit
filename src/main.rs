use crate::commands::download::download_history;
use crate::commands::get_quote::get_quote;
use clap::Parser;
use anyhow::Result;
use crate::downloader::decode::stream_decode_and_write;

mod client;
mod commands;
mod downloader;
mod types;
//use crate::commands::download::download_and_save;


//
// #[tokio::main]
// async fn main() -> databento::Result<()> {
//     //download_and_save("CLN3","CLN3_2023-06").await.expect("Couldnt complete");
//     download_history().await.expect("Couldn't finish retrieving history");
//     //get_quote().await.expect("Couldn't get quote");
//     Ok(())
// }



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
        Task::Download => download_history(base_path).await?,
        Task::Quote => get_quote().await?,
        Task::Decode => stream_decode_and_write(base_path).await?,
    }

    Ok(())
}