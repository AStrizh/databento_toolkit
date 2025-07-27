use std::{fs::File, io::BufWriter};
use tokio::fs::File as TokioFile;

use tokio::io::{BufReader, AsyncRead};
use std::pin::Pin;

use databento::{
    dbn::{OhlcvMsg, Schema},
    historical::timeseries::{AsyncDbnDecoder, GetRangeToFileParams},
};
use serde::Serialize;
use time::macros::datetime;

use crate::client::get_client;

use tokio::io::{BufReader as AsyncBufReader};
use async_compression::tokio::bufread::ZstdDecoder;
use std::io::Write;

const DBN_PATH: &str = "CLN3_2023-06.dbn.zst";
const JSON_PATH: &str = "CLN3_2023-06_ohlcv1m.json";
const DBN_EXT: &str = ".dbn.zst";
const JSON_EXT: &str = "_ohlcv1m.json";
#[derive(Serialize)]
pub struct JsonOhlcv {
    instrument_name: String,
    instrument_id: u32,
    ts_event: u64,
    open: i64,
    high: i64,
    low: i64,
    close: i64,
    volume: u64,
}


/// Manages the downloading of records from Databento and unpacking into JSON
pub async fn download_and_save(symbol: &str, base_path: &str) -> databento::Result<()> {
    println!("Starting download...");
    download_data(symbol, base_path).await?;
    println!("Download complete. Reading file...");


    // let records = read_dbn_to_json_records(base_path, symbol).await?;
    // println!("Parsed {} records", records.len());

    stream_decode_and_write(base_path, symbol).await.expect("Could not complete decode and save.");
    println!("Saved JSON to {base_path}{JSON_EXT}");

    Ok(())
}


/// Downloads the data and stores it in memory
async fn download_data(symbol: &str, base_path: &str ) -> databento::Result<()> {
    let mut client = get_client();
    let path = format!("{base_path}{DBN_EXT}");

    client
        .timeseries()
        .get_range_to_file(
            &GetRangeToFileParams::builder()
                .dataset("GLBX.MDP3")
                .date_time_range((
                    datetime!(2023-06-01 00:00 UTC),
                    datetime!(2023-06-30 23:59 UTC),
                ))
                .symbols(symbol.parse::<String>().unwrap())
                .schema(Schema::Ohlcv1M)
                .path(path)
                .build(),
        )
        .await?;

    Ok(())
}

/// Decodes and writes records from the `.dbn.zst ` file directly to `.json`, one per line
async fn stream_decode_and_write(base_path: &str, symbol: &str) -> databento::Result<()> {

    let input_path = format!("{base_path}{}", DBN_EXT);
    let output_path = format!("{base_path}{}", JSON_EXT);

    let file = TokioFile::open(input_path).await?;
    let buf_reader = AsyncBufReader::new(file);
    let zstd_decoder = ZstdDecoder::new(buf_reader);
    let pinned_reader = Box::pin(zstd_decoder) as Pin<Box<dyn AsyncRead + Send>>;

    let mut decoder = AsyncDbnDecoder::new(pinned_reader).await?;

    let out_file = File::create(output_path).expect("Failed to create output JSON file");
    let mut writer = BufWriter::new(out_file);

    while let Some(msg) = decoder.decode_record::<OhlcvMsg>().await? {
        let record = JsonOhlcv {
            instrument_name: symbol.to_string(),
            instrument_id: msg.hd.instrument_id,
            ts_event: msg.hd.ts_event,
            open: msg.open,
            high: msg.high,
            low: msg.low,
            close: msg.close,
            volume: msg.volume,
        };

        serde_json::to_writer(&mut writer, &record).expect("Failed to write JSON record");
        writeln!(&mut writer).expect("Failed to write newline");
    }

    Ok(())
}





