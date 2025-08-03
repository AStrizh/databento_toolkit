use databento::dbn::Schema;
use databento::historical::timeseries::GetRangeToFileParams;
use crate::types::DownloadTask;

// In normal builds, use the real downloader, uncomment below this line


// pub async fn download_data(mut task: DownloadTask) -> databento::Result<()> {
//     let path = format!("{}/{}_{}_{}.dbn.zst", task.base_path, task.start, task.end, task.symbol);
//
//     task.client
//         .get_mut()
//         .timeseries()
//         .get_range_to_file(
//             &GetRangeToFileParams::builder()
//                 .dataset("GLBX.MDP3")
//                 .date_time_range((task.start.midnight().assume_utc(), task.end.midnight().assume_utc()))
//                 .symbols(task.symbol.clone())
//                 .schema(Schema::Ohlcv1M)
//                 .path(path)
//                 .build(),
//         )
//         .await?;
//
//     println!("Finished downloading {} for period {} to {}", task.symbol, task.start, task.end);
//     Ok(())
// }



// --- TEMPORARY FOR INTEGRATION TESTING ---
// Comment this code for real builds
pub async fn download_data(task: DownloadTask) -> anyhow::Result<()> {
    use std::fs;
    use std::path::Path;

    let filename = format!("{}/{}_{}_{}.mock", task.base_path, task.start, task.end, task.symbol);
    fs::create_dir_all(Path::new(&task.base_path))?;
    fs::write(&filename, b"mock data")?;
    Ok(())
}
