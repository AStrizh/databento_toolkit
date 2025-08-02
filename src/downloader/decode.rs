use async_compression::tokio::bufread::ZstdDecoder;
use databento::dbn::{decode::AsyncDbnDecoder, OhlcvMsg};
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    pin::Pin,
};
use tokio::{
    fs::File as TokioFile,
    io::{AsyncRead, BufReader as AsyncBufReader},
};

use crate::types::JsonOhlcv;

const DBN_EXT: &str = ".dbn.zst";
const JSON_EXT: &str = "_ohlcv1m.json";

/// Recursively decode all `.dbn.zst` files in a directory tree to JSON files.
pub async fn decode_all_in_dir(root_dir: &str) -> databento::Result<()> {
    let mut stack = vec![PathBuf::from(root_dir)];

    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("zst")
                && path.file_name().and_then(|f| f.to_str()).map_or(false, |f| f.ends_with(DBN_EXT))
            {
                let base_path = path.with_extension(""); // removes only `.zst`
                let base_path = base_path
                    .to_str()
                    .unwrap()
                    .strip_suffix(".dbn")
                    .unwrap()
                    .to_string();

                if let Err(e) = stream_decode_and_write(&base_path).await {
                    eprintln!("Error decoding file {}: {:?}", path.display(), e);
                }
            }
        }
    }

    Ok(())
}

/// Decode a single `.dbn.zst` file to `<base>_ohlcv1m.json`.
pub async fn stream_decode_and_write(base_path: &str) -> databento::Result<()> {
    let input_path = format!("{base_path}{DBN_EXT}");
    let output_path = format!("{base_path}{JSON_EXT}");

    // Get filename without path and extension as instrument name
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

    let out_file = File::create(&output_path)?;
    let mut writer = BufWriter::new(out_file);

    println!("Decoding {input_path} → {output_path}");

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

        serde_json::to_writer(&mut writer, &record).expect("Could not decode");
        writeln!(&mut writer)?;
    }

    Ok(())
}
