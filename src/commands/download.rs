use std::fs;
use std::path::Path;
use std::sync::Arc;
use crate::downloader::{contracts::generate_cl_contract_periods, fetch::download_data};
use crate::client::DBClient;
use crate::types::DownloadTask;
use tokio::sync::Semaphore;

use anyhow::Result;


pub async fn download_history(base_path: &str) -> Result<()> {
    let periods = generate_cl_contract_periods(2023, 2024);

    for (_, start, _) in &periods {
        let year_path = format!("{base_path}/{}/", start.year());

        if !Path::new(&year_path).exists() {
            fs::create_dir_all(&year_path)?;
        }
    }

    // Semaphore with a limit of 5 concurrent tasks
    let semaphore = Arc::new(Semaphore::new(10));

    let handles = periods
        .into_iter()
        .map(|(symbol, start, end)| {
            let symbol_path = format!("{base_path}/{}/", start.year());
            let task = DownloadTask {
                client: DBClient::new(),
                symbol,
                base_path: symbol_path,
                start,
                end,
            };

            let semaphore = Arc::clone(&semaphore);

            tokio::spawn(async move {
                // Acquire a permit to ensure only `N` tasks run concurrently
                let _permit = semaphore.acquire().await;

                // Perform the download
                download_data(task).await
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        handle.await??;
    }

    Ok(())
}