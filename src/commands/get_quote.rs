use anyhow::{Context, Result};
use databento::{dbn::Schema, historical::metadata::GetCostParams};
use std::{fs::File, io::Write, path::Path, sync::Arc};
use time::Date;
use tokio::{sync::Semaphore, task::JoinSet};

use crate::client::DBClient;
use crate::downloader::contracts::generate_contract_periods;
use crate::downloader::range::download_time_range;

#[derive(Debug, Clone)]
pub struct QuoteRequest {
    pub dataset: String,
    pub symbol: String,
    pub schema: Schema,
    pub start: Date,
    pub end: Date,
}

impl QuoteRequest {
    pub fn new(
        dataset: impl Into<String>,
        symbol: impl Into<String>,
        schema: Schema,
        start: Date,
        end: Date,
    ) -> Self {
        Self {
            dataset: dataset.into(),
            symbol: symbol.into(),
            schema,
            start,
            end,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HistoryQuoteEstimate {
    pub total_cost_usd: f64,
    pub successful_count: usize,
    pub total_count: usize,
    pub failed_contracts: Vec<FailedContractEstimate>,
}

#[derive(Debug, Clone)]
pub struct FailedContractEstimate {
    pub contract_symbol: String,
    pub start: Date,
    pub end: Date,
    pub api_request: String,
    pub error_message: String,
}

#[derive(Debug, Clone)]
struct ContractQuoteRequest {
    symbol: String,
    start: Date,
    end: Date,
}

const ESTIMATE_CONCURRENCY_LIMIT: usize = 10;
pub const ERROR_REPORT_PATH: &str = "error_response.txt";

/// Calls Databento metadata.get_cost using request parameters.
/// This is intentionally reusable so the GUI can call it later.
pub async fn estimate_quote_cost(request: &QuoteRequest) -> databento::Result<f64> {
    let mut client = DBClient::new();
    let (start_dt, end_dt) = download_time_range(request.start, request.end);

    client
        .get_mut()
        .metadata()
        .get_cost(
            &GetCostParams::builder()
                .dataset(request.dataset.as_str())
                .date_time_range((start_dt, end_dt))
                .symbols(request.symbol.as_str())
                .schema(request.schema)
                .build(),
        )
        .await
}

/// Estimate total cost for the same contract-period requests used by `download_history`.
/// This does not download any data; it only queries Databento metadata pricing.
pub async fn estimate_download_history_cost(
    start_date: Date,
    end_date: Date,
    base_symbols: &[&str],
    dataset: &str,
    schema: Schema,
) -> Result<HistoryQuoteEstimate> {
    let requests = build_contract_quote_requests(start_date, end_date, base_symbols);
    let total_count = requests.len();
    let semaphore = Arc::new(Semaphore::new(ESTIMATE_CONCURRENCY_LIMIT));
    let mut join_set = JoinSet::new();
    let mut total_cost_usd = 0.0;
    let mut successful_count = 0usize;
    let mut failed_contracts = Vec::new();

    for request in requests {
        let semaphore = Arc::clone(&semaphore);
        let dataset = dataset.to_string();
        join_set.spawn(async move {
            let _permit = semaphore
                .acquire_owned()
                .await
                .context("Failed to acquire estimate semaphore permit")?;
            let api_request = build_api_request_string(&request, &dataset, schema);
            let estimate_result = estimate_single_contract_cost(&request, &dataset, schema).await;
            Ok::<(ContractQuoteRequest, String, databento::Result<f64>), anyhow::Error>((
                request,
                api_request,
                estimate_result,
            ))
        });
    }

    while let Some(join_result) = join_set.join_next().await {
        let (request, api_request, task_result) =
            join_result.context("Estimate task failed to join")??;
        match task_result {
            Ok(cost) => {
                total_cost_usd += cost;
                successful_count += 1;
            }
            Err(error) => {
                failed_contracts.push(FailedContractEstimate {
                    contract_symbol: request.symbol,
                    start: request.start,
                    end: request.end,
                    api_request,
                    error_message: error.to_string(),
                });
            }
        }
    }

    Ok(HistoryQuoteEstimate {
        total_cost_usd,
        successful_count,
        total_count,
        failed_contracts,
    })
}

fn build_contract_quote_requests(
    start_date: Date,
    end_date: Date,
    base_symbols: &[&str],
) -> Vec<ContractQuoteRequest> {
    let mut requests = Vec::new();

    for &base_symbol in base_symbols {
        for (contract_symbol, contract_start, contract_end) in
            generate_contract_periods(base_symbol, start_date, end_date)
        {
            requests.push(ContractQuoteRequest {
                symbol: contract_symbol,
                start: contract_start,
                end: contract_end,
            });
        }
    }

    requests
}

async fn estimate_single_contract_cost(
    request: &ContractQuoteRequest,
    dataset: &str,
    schema: Schema,
) -> databento::Result<f64> {
    let quote_request = QuoteRequest::new(
        dataset.to_string(),
        request.symbol.clone(),
        schema,
        request.start,
        request.end,
    );
    estimate_quote_cost(&quote_request).await
}

fn build_api_request_string(request: &ContractQuoteRequest, dataset: &str, schema: Schema) -> String {
    let (start_dt, end_dt) = download_time_range(request.start, request.end);
    format!(
        "POST metadata.get_cost dataset={dataset} schema={schema} symbols={} stype_in=raw_symbol start={start_dt} end={end_dt}",
        request.symbol
    )
}

pub fn write_estimate_error_report(path: impl AsRef<Path>, estimate: &HistoryQuoteEstimate) -> Result<()> {
    let path = path.as_ref();
    let mut file = File::create(path)
        .with_context(|| format!("Failed to create error report file: {}", path.display()))?;

    writeln!(
        file,
        "Total estimated cost: ${:.4} ({} successful contracts)",
        estimate.total_cost_usd, estimate.successful_count
    )
    .context("Failed to write estimate summary")?;
    writeln!(
        file,
        "Failed contracts: {} out of {}",
        estimate.failed_contracts.len(),
        estimate.total_count
    )
    .context("Failed to write estimate failure counts")?;

    if estimate.failed_contracts.is_empty() {
        writeln!(file, "Failed contract symbols: none")
            .context("Failed to write no-failure marker")?;
        return Ok(());
    }

    let failed_symbols = estimate
        .failed_contracts
        .iter()
        .map(|failure| failure.contract_symbol.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    writeln!(file, "Failed contract symbols: {failed_symbols}")
        .context("Failed to write failed symbol list")?;
    writeln!(file).context("Failed to write spacing")?;

    for failure in &estimate.failed_contracts {
        writeln!(
            file,
            "[{} | {} to {}]",
            failure.contract_symbol, failure.start, failure.end
        )
        .context("Failed to write failure contract header")?;
        writeln!(file, "API request: {}", failure.api_request)
            .context("Failed to write failed API request")?;
        writeln!(file, "Error: {}", failure.error_message)
            .context("Failed to write API error details")?;
        writeln!(file).context("Failed to write spacing")?;
    }

    Ok(())
}
