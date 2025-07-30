use crate::commands::download::download_history;

mod client;
mod commands;
mod downloader;
mod types;
//use crate::commands::download::download_and_save;



#[tokio::main]
async fn main() -> databento::Result<()> {
    //download_and_save("CLN3","CLN3_2023-06").await.expect("Couldnt complete");
    download_history().await.expect("Couldn't finish retrieving history");
    Ok(())
}