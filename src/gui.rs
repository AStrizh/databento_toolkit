use crate::commands::download::download_history;
use crate::downloader::decode::decode_all_in_dir;
use crate::types::JsonOhlcv;
use anyhow::{Context, Result};
use eframe::{egui, App};
//use egui_extras::DatePickerButton;
use crate::custom_datepicker::CustomDatePickerButton as DatePickerButton;

use std::sync::{Arc, Mutex};
use time::{Date, Month};
use chrono::{NaiveDate, Datelike};

// ───── Constants ─────
const SUPPORTED_SYMBOLS: &[&str] = &["CL", "NG", "ES", "NQ", "RTY", "YM"];

// ───── Helper Functions ─────
/// Convert `time::Date` to `chrono::NaiveDate`.
fn time_to_naive_date(date: Date) -> NaiveDate {
    NaiveDate::from_ymd_opt(date.year(), date.month() as u32, date.day().into()).unwrap()
}

/// Convert `chrono::NaiveDate` to `time::Date`.
fn naive_date_to_time(date: NaiveDate) -> Result<Date> {
    Date::from_calendar_date(date.year(), Month::try_from(date.month() as u8)?, date.day() as u8)
        .context("Invalid NaiveDate to time::Date conversion")
}

// ───── GUI App State ─────
pub(crate) struct AppState {
    start_date: NaiveDate,
    end_date: NaiveDate,
    selected_symbols: Vec<bool>,
    task_status: Arc<Mutex<String>>,
    runtime: tokio::runtime::Runtime,
}

impl Default for AppState {
    fn default() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create runtime");

        Self {
            start_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            selected_symbols: SUPPORTED_SYMBOLS.iter().map(|_| false).collect(),
            task_status: Arc::new(Mutex::new(String::new())),
            runtime,
        }
    }
}

impl App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Databento Toolkit GUI");

            // Date Pickers
            ui.horizontal(|ui| {
                ui.label("Start Date:");
                let response = ui.add(DatePickerButton::new(&mut self.start_date).id_source("start_date"));
                if response.changed() {
                    match naive_date_to_time(self.start_date) {
                        Ok(converted_date) => *self.task_status.lock().unwrap() = format!("Start date updated: {converted_date}"),
                        Err(e) => *self.task_status.lock().unwrap() = format!("Error: {e}"),
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("End Date:");
                let response = ui.add(DatePickerButton::new(&mut self.end_date).id_source("end_date"));
                if response.changed() {
                    match naive_date_to_time(self.end_date) {
                        Ok(converted_date) => *self.task_status.lock().unwrap() = format!("End date updated: {converted_date}"),
                        Err(e) => *self.task_status.lock().unwrap() = format!("Error: {e}"),
                    }
                }
            });


            // Symbol selection checkboxes
            ui.label("Select Symbols:");
            for (i, &symbol) in SUPPORTED_SYMBOLS.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.selected_symbols[i], symbol);
                });
            }

            let status_arc = self.task_status.clone();

            if ui.button("Download History").clicked() {
                // Convert `NaiveDate` to `Date`.
                let start_date = match naive_date_to_time(self.start_date) {
                    Ok(date) => date,
                    Err(e) => {
                        *status_arc.lock().unwrap() = format!("Start date error: {e}");
                        return;
                    }
                };
                let end_date = match naive_date_to_time(self.end_date) {
                    Ok(date) => date,
                    Err(e) => {
                        *status_arc.lock().unwrap() = format!("End date error: {e}");
                        return;
                    }
                };

                let symbols: Vec<_> = SUPPORTED_SYMBOLS
                    .iter()
                    .zip(self.selected_symbols.iter())
                    .filter_map(|(&symbol, &checked)| if checked { Some(symbol) } else { None })
                    .collect();

                if symbols.is_empty() {
                    *status_arc.lock().unwrap() = "No symbols selected.".to_string();
                    return;
                }

                *status_arc.lock().unwrap() = "Downloading...".to_string();

                let status_arc_inner = status_arc.clone();
                self.runtime.spawn(async move {
                    let result = download_history(
                        start_date,
                        end_date,
                        &symbols.iter().map(AsRef::as_ref).collect::<Vec<_>>(),
                        "Hist_Fut_Data",
                    )
                    .await;

                    let mut status = status_arc_inner.lock().unwrap();
                    *status = match result {
                        Ok(_) => "Download complete".to_string(),
                        Err(e) => format!("Error: {}", e),
                    };
                });
            }

            if ui.button("Decode Files").clicked() {
                *status_arc.lock().unwrap() = "Decoding...".to_string();
                let status_arc_inner = status_arc.clone();
                self.runtime.spawn(async move {
                    let result = decode_all_in_dir("Hist_Fut_Data").await;
                    let mut status = status_arc_inner.lock().unwrap();
                    *status = match result {
                        Ok(_) => "Decoding complete".to_string(),
                        Err(e) => format!("Decode error: {}", e),
                    };
                });
            }

            ui.label(&*self.task_status.lock().unwrap());
        });
    }
}