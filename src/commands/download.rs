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
use crate::client::get_client;





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

use crate::downloader::{contracts::generate_contract_periods, fetch::download_data, decode::stream_decode_and_write};
use time::macros::date;

pub async fn download_history() -> databento::Result<()> {
    let periods = generate_contract_periods(date!(2015 - 01 - 01), date!(2025 - 07 - 25));

    for (symbol, start_date, end_date) in periods {
        let year_folder = format!("data/{}", start_date.year());
        std::fs::create_dir_all(&year_folder)?;

        let base_path = format!("{}/{}", year_folder, symbol);

        download_data(&symbol, &base_path, start_date, end_date).await?;
        stream_decode_and_write(&base_path, &symbol).await?;
    }

    Ok(())
}
