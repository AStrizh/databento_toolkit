use databento::dbn::Schema;
use databento::historical::timeseries::GetRangeToFileParams;
use crate::types::DownloadTask;

pub async fn download_data(mut task: DownloadTask) -> databento::Result<()> {
    let path = format!("{}/{}.dbn.zst", task.base_path, task.symbol);
    task.client
        .get_mut()
        .timeseries()
        .get_range_to_file(
            &GetRangeToFileParams::builder()
                .dataset("GLBX.MDP3")
                .date_time_range((task.start.midnight().assume_utc(), task.end.midnight().assume_utc()))
                .symbols(task.symbol.clone())
                .schema(Schema::Ohlcv1M)
                .path(path)
                .build(),
        )
        .await?;

    println!("Finished downloading {} for period {} to {}", task.symbol, task.start, task.end);
    Ok(())
}