mod client;
mod commands;

use crate::commands::get_cost::run_get_cost;

#[tokio::main]
async fn main() -> databento::Result<()> {
    run_get_cost().await?;
    Ok(())
}