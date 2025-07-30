use serde::{Deserialize, Serialize};

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