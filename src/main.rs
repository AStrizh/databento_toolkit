mod client;
mod commands;
use crate::commands::download::download_and_save;



#[tokio::main]
async fn main() -> databento::Result<()> {
    download_and_save("CLN3","CLN3_2023-06").await.expect("Couldnt complete");
    Ok(())
}