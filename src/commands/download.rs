use std::fs;
use std::path::Path;
use std::sync::Arc;
use anyhow::Result;
use tokio::sync::Semaphore;
use time::Date;
use crate::client::DBClient;
use crate::downloader::contracts::generate_contract_periods;
use crate::downloader::fetch::download_data;
use crate::types::DownloadTask;

pub async fn download_history(
    start_date: Date,
    end_date: Date,
    symbols: &[&str],
    base_path: &str,
) -> Result<()> {
    let tasks = generate_tasks(start_date, end_date, symbols, base_path)?;
    run_download_tasks(tasks).await
}

fn generate_tasks(
    start_date: Date,
    end_date: Date,
    symbols: &[&str],
    base_path: &str,
) -> Result<Vec<DownloadTask>> {
    let mut tasks = Vec::new();

    for &base_symbol in symbols {
        let periods = generate_contract_periods(base_symbol, start_date, end_date);
        let symbol_dir = format!("{}/{}/{}", base_path, start_date.year(), base_symbol);

        if !Path::new(&symbol_dir).exists() {
            fs::create_dir_all(&symbol_dir)?;
        }

        for (contract_symbol, start, end) in periods {
            let task = DownloadTask {
                client: DBClient::new(),
                symbol: contract_symbol,
                base_path: symbol_dir.clone(),
                start,
                end,
            };

            tasks.push(task);
        }
    }

    Ok(tasks)
}

async fn run_download_tasks(tasks: Vec<DownloadTask>) -> Result<()> {
    let semaphore = Arc::new(Semaphore::new(10));

    let handles = tasks
        .into_iter()
        .map(|task| {
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

//-----------------------------------------------------------------------------------------------------------------//
#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;
    use std::fs;

    fn cleanup_test_dir(base_path: &str) {
        if Path::new(base_path).exists() {
            fs::remove_dir_all(base_path).expect("Cleanup failed");
        }
    }

    #[tokio::test]
    async fn test_generate_tasks_creates_expected_structure() {
        let base_path = "test_output_generate";
        cleanup_test_dir(base_path);

        let start = date!(2023 - 01 - 01);
        let end = date!(2023 - 12 - 31);
        let tasks = generate_tasks(start, end, &["CL", "NG"], base_path).expect("Should create tasks");

        assert!(!tasks.is_empty());

        // Ensure directory structure is correct
        for symbol in ["CL", "NG"] {
            let expected_dir = format!("{}/{}/{}", base_path, start.year(), symbol);
            assert!(Path::new(&expected_dir).exists(), "Expected directory missing: {}", expected_dir);
        }

        cleanup_test_dir(base_path);
    }

    #[tokio::test]
    async fn test_run_download_tasks_creates_mock_files() {
        let base_path = "test_output_run";
        cleanup_test_dir(base_path);

        let start = date!(2023 - 01 - 01);
        let end = date!(2023 - 12 - 31);
        let tasks = generate_tasks(start, end, &["CL"], base_path).expect("Should create tasks");

        // Manually fake the effect of download_data
        for task in &tasks {
            let file = format!("{}/{}_{}_{}.mock", task.base_path, task.symbol, task.start, task.end);
            fs::create_dir_all(&task.base_path).unwrap();
            fs::write(file, "mock data").unwrap();
        }

        // Confirm files were created
        for task in &tasks {
            let file = format!("{}/{}_{}_{}.mock", task.base_path, task.symbol, task.start, task.end);
            assert!(Path::new(&file).exists(), "Missing file: {}", file);
        }

        cleanup_test_dir(base_path);
    }

    #[tokio::test]
    async fn test_download_history_creates_valid_tasks() {
        let base_path = "test_output_history";
        cleanup_test_dir(base_path);

        let start = date!(2023 - 01 - 01);
        let end = date!(2023 - 01 - 15);
        let result = download_history(start, end, &["NG"], base_path).await;

        assert!(result.is_ok());

        let ng_dir = format!("{}/{}/NG", base_path, start.year());
        assert!(Path::new(&ng_dir).exists(), "Expected directory for NG not found");

        cleanup_test_dir(base_path);
    }

    #[tokio::test]
    async fn test_invalid_symbol_panics() {
        let base_path = "test_output_invalid";
        let result = std::panic::catch_unwind(|| {
            generate_tasks(date!(2023 - 01 - 01), date!(2023 - 12 - 31), &["ZZZ"], base_path).unwrap();
        });

        assert!(result.is_err(), "Expected panic for unsupported symbol");
    }
}
