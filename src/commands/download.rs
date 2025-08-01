
use crate::downloader::{contracts::generate_cl_contract_periods, fetch::download_data};
use crate::client::DBClient;
use crate::types::DownloadTask;

use anyhow::Result;

pub async fn download_history(base_path: &str) -> Result<()> {
    let periods = generate_cl_contract_periods(2023, 2024);

    //base_path: &str, symbol: &str

    let handles = periods
        .into_iter()
        .map(|(symbol, start, end)| {
            let base_path = format!("{base_path}/{}/", start.year());
            let task = DownloadTask {
                client: DBClient::new(),
                symbol,
                base_path,
                start,
                end,
            };

            tokio::spawn(async move {
                download_data(task).await
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        handle.await??;
    }

    Ok(())
}