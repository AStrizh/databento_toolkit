use async_compression::tokio::bufread::ZstdDecoder;
use databento::dbn::decode::AsyncDbnDecoder;
use databento::dbn::OhlcvMsg;
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
    pin::Pin,
};
use std::path::PathBuf;
use tokio::{
    fs::File as TokioFile,
    io::{AsyncRead, BufReader as AsyncBufReader},
};
use crate::types::JsonOhlcv;

const DBN_EXT: &str = ".dbn.zst";
const JSON_EXT: &str = "_ohlcv1m.json";

/// Decodes all `.dbn.zst` files in the given directory tree.
pub async fn decode_all_in_dir(root_dir: &str) -> databento::Result<()> {
    let mut stack = vec![PathBuf::from(root_dir)];

    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                stack.push(path);
            } else if let Some(ext) = path.extension() {
                if ext == "zst" && path.to_str().unwrap().ends_with(".dbn.zst") {
                    let base_path = path.to_str().unwrap().strip_suffix(".dbn.zst").unwrap();
                    stream_decode_and_write(base_path).await?;
                }
            }
        }
    }

    Ok(())
}

/// Decodes and writes records from a `.dbn.zst` file to JSON.
pub async fn stream_decode_and_write(base_path: &str) -> databento::Result<()> {
    let input_path = format!("{base_path}{DBN_EXT}");
    let output_path = format!("{base_path}{JSON_EXT}");

    // Try to infer the symbol from the filename
    let symbol = Path::new(base_path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let file = TokioFile::open(&input_path).await?;
    let buf_reader = AsyncBufReader::new(file);
    let zstd_decoder = ZstdDecoder::new(buf_reader);
    let pinned_reader = Box::pin(zstd_decoder) as Pin<Box<dyn AsyncRead + Send>>;

    let mut decoder = AsyncDbnDecoder::new(pinned_reader).await?;

    let out_file = File::create(&output_path).expect("Failed to create output JSON file");
    let mut writer = BufWriter::new(out_file);

    while let Some(msg) = decoder.decode_record::<OhlcvMsg>().await? {
        let record = JsonOhlcv {
            instrument_name: symbol.clone(),
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
