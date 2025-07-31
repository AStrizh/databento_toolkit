use serde::{Deserialize, Serialize};
use time::Date;
use crate::client::DBClient;

#[derive(Serialize, Deserialize)]
pub struct JsonOhlcv {
    pub instrument_name: String,
    pub instrument_id: u32,
    pub ts_event: u64,
    pub open: i64,
    pub high: i64,
    pub low: i64,
    pub close: i64,
    pub volume: u64,
}


#[derive(Clone)]
pub struct DownloadTask {
    pub client: DBClient,
    pub symbol: String,
    pub base_path: String,
    pub start: Date,
    pub end: Date,
}
