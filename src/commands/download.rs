use std::fs;
use std::path::Path;
use std::sync::Arc;
use crate::downloader::{contracts::generate_contract_periods, fetch::download_data};
use crate::client::DBClient;
use crate::types::DownloadTask;
use tokio::sync::Semaphore;
use anyhow::Result;

use time::Date;

pub async fn download_history(
    start_date: Date,
    end_date: Date,
    symbols: &[&str],
    base_path: &str
) -> Result<()> {
    let periods = generate_contract_periods(start_date, end_date, symbols);

    // Create symbol directories inside each year
    for (symbol, start, _) in &periods {
        let symbol_dir = format!("{}/{}/{}", base_path, start.year(), symbol);
        if !Path::new(&symbol_dir).exists() {
            fs::create_dir_all(&symbol_dir)?;
        }
    }

    let semaphore = Arc::new(Semaphore::new(10));

    let handles = periods
        .into_iter()
        .map(|(symbol, start, end)| {
            let symbol_path = format!("{}/{}/{}", base_path, start.year(), symbol);

            let task = DownloadTask {
                client: DBClient::new(),
                symbol,
                base_path: symbol_path,
                start,
                end,
            };

            let semaphore = Arc::clone(&semaphore);
            tokio::spawn(async move {
                let _permit = semaphore.acquire().await;
                download_data(task).await
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        handle.await??;
    }

    Ok(())
}
