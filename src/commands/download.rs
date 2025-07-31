// ───── Std imports ─────
use std::{
    fs::File,
    io::{BufWriter, Write},
    pin::Pin,
};

// ───── Tokio imports ─────
use tokio::{
    fs::File as TokioFile,
    io::{AsyncRead, BufReader as AsyncBufReader},
};

// ───── Third-party crates ─────
use async_compression::tokio::bufread::ZstdDecoder;
use databento::{
    dbn::{OhlcvMsg, Schema},
    historical::timeseries::{AsyncDbnDecoder, GetRangeToFileParams},
};
use serde::Serialize;
use time::Date;
use time::macros::datetime;

// ───── Internal modules ─────
//use crate::client::get_client;





// /// Manages the downloading of records from Databento and unpacking into JSON
// pub async fn download_and_save(symbol: &str, base_path: &str) -> databento::Result<()> {
//     println!("Starting download...");
//     download_data(symbol, base_path).await?;
//     println!("Download complete. Reading file...");
//
//     stream_decode_and_write(base_path, symbol).await.expect("Could not complete decode and save.");
//     println!("Saved JSON to {base_path}{JSON_EXT}");
//
//     Ok(())
// }
//
//
// /// Downloads the data and stores it in memory
// async fn download_data(symbol: &str, base_path: &str ) -> databento::Result<()> {
//     let mut client = get_client();
//     let path = format!("{base_path}{DBN_EXT}");
//
//     client
//         .timeseries()
//         .get_range_to_file(
//             &GetRangeToFileParams::builder()
//                 .dataset("GLBX.MDP3")
//                 .date_time_range((
//                     datetime!(2023-06-01 00:00 UTC),
//                     datetime!(2023-06-30 23:59 UTC),
//                 ))
//                 .symbols(symbol.parse::<String>().unwrap())
//                 .schema(Schema::Ohlcv1M)
//                 .path(path)
//                 .build(),
//         )
//         .await?;
//
//     Ok(())
// }
//



//-------------------------------------------------------------------------------------------------//
//TODO: Check if the following ChatGPT code is any good

use crate::downloader::{contracts::generate_cl_contract_periods, fetch::download_data, decode::stream_decode_and_write};
use time::macros::date;
use crate::client::DBClient;
use crate::types::DownloadTask;

use anyhow::Result;

pub async fn download_history() -> Result<()> {
    let periods = generate_cl_contract_periods(2015, 2025);

    let handles = periods
        .into_iter()
        .map(|(symbol, start, end)| {
            let base_path = format!("data/{}/{}", start.year(), &symbol);
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



fn create_year_folders(years: &[i32]) -> std::io::Result<()> {
    for &year in years {
        let path = format!("data/{}", year);
        std::fs::create_dir_all(&path)?;
    }
    Ok(())
}


// let periods = generate_cl_contract_periods(2020,2025);
//
// for (symbol, start_date, end_date) in periods {
//     let year_folder = format!("data/{}", start_date.year());
//     std::fs::create_dir_all(&year_folder)?;
//
//     let base_path = format!("{}/{}", year_folder, symbol);
//
//     download_data(&symbol, &base_path, start_date, end_date).await?;
//     stream_decode_and_write(&base_path, &symbol).await?;
// }




// pub async fn download_history() -> databento::Result<()> {
//     let start_year = 2015;
//     let end_year = 2025;
//
//     let periods = generate_contract_windows(start_year, end_year);
//     create_year_folders(&(start_year..=end_year).collect::<Vec<_>>())?;
//
//     for (symbol, start_date, end_date) in periods {
//         let base_path = format!("data/{}/{}", start_date.year(), symbol);
//
//         download_data(&symbol, &base_path, start_date, end_date).await?;
//         stream_decode_and_write(&base_path, &symbol).await?;
//     }
//
//     Ok(())
// }
