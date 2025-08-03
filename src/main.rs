use crate::commands::download::download_history;
use crate::commands::get_quote::get_quote;
use crate::downloader::decode::decode_all_in_dir;
use crate::types::JsonOhlcv;
use anyhow::Result;
use eframe::{egui, App};
use std::sync::{Arc, Mutex};
use time::{Date};

mod client;
mod commands;
mod downloader;
mod types;
mod gui;

mod custom_datepicker;

use crate::custom_datepicker::CustomDatePickerButton;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Databento Toolkit",
        options,
        Box::new(|_cc| Ok(Box::new(gui::AppState::default()))),
    )
}