mod client;
mod commands;

use crate::commands::download::download_and_save;
use crate::commands::get_quote::get_quote;

#[tokio::main]
async fn main() -> databento::Result<()> {
    download_and_save("CLN3","CLN3_2023-06").await.expect("Couldnt complete");
    Ok(())
}