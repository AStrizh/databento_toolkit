
use async_compression::tokio::bufread::ZstdDecoder;
use databento::dbn::decode::AsyncDbnDecoder;
use databento::dbn::OhlcvMsg;

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
use crate::types::JsonOhlcv;

const DBN_EXT: &str = ".dbn.zst";
const JSON_EXT: &str = "_ohlcv1m.json";


/// Decodes and writes records from the `.dbn.zst ` file directly to `.json`, one per line
pub(crate) async fn stream_decode_and_write(base_path: &str) -> databento::Result<()> {

    //TODO:Paths are not correct, fix them
    let input_path = format!("{base_path}{}", DBN_EXT);
    let output_path = format!("{base_path}{}", JSON_EXT);
    let symbol = "CL"; //TODO: Replace with code that detects symbol name from .dbn.zst file

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