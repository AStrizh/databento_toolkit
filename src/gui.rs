use crate::commands::download::download_history;
use crate::commands::get_quote::{
    estimate_download_history_cost,
    write_estimate_error_report,
    ERROR_REPORT_PATH,
};
use crate::downloader::decode::decode_all_in_dir;
use databento::dbn::Schema;
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
    cost_estimate: Arc<Mutex<String>>,
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
            cost_estimate: Arc::new(Mutex::new("No estimate yet".to_string())),
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
                let response = ui.add(DatePickerButton::new(&mut self.start_date).id_salt("start_date"));
                if response.changed() {
                    match naive_date_to_time(self.start_date) {
                        Ok(converted_date) => *self.task_status.lock().unwrap() = format!("Start date updated: {converted_date}"),
                        Err(e) => *self.task_status.lock().unwrap() = format!("Error: {e}"),
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("End Date:");
                let response = ui.add(DatePickerButton::new(&mut self.end_date).id_salt("end_date"));
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
            let cost_arc = self.cost_estimate.clone();

            if ui.button("Estimate Cost").clicked() {
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

                *status_arc.lock().unwrap() = "Estimating cost...".to_string();
                *cost_arc.lock().unwrap() = "Estimating...".to_string();

                let status_arc_inner = status_arc.clone();
                let cost_arc_inner = cost_arc.clone();
                self.runtime.spawn(async move {
                    let result = estimate_download_history_cost(
                        start_date,
                        end_date,
                        &symbols,
                        "GLBX.MDP3",
                        Schema::Ohlcv1M,
                    )
                    .await;

                    match result {
                        Ok(estimate) => {
                            let failed_count = estimate.failed_contracts.len();
                            let failed_symbols = if failed_count == 0 {
                                "none".to_string()
                            } else {
                                estimate
                                    .failed_contracts
                                    .iter()
                                    .map(|failure| failure.contract_symbol.clone())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            };

                            *cost_arc_inner.lock().unwrap() = format!(
                                "Total estimated cost: ${:.4} ({} successful contracts)\nFailed contracts: {} out of {}\nFailed contract symbols: {}",
                                estimate.total_cost_usd,
                                estimate.successful_count,
                                failed_count,
                                estimate.total_count,
                                failed_symbols
                            );

                            let mut status = if failed_count == 0 {
                                "Cost estimate complete".to_string()
                            } else {
                                format!(
                                    "Cost estimate complete with failures. See {}",
                                    ERROR_REPORT_PATH
                                )
                            };

                            if let Err(report_error) =
                                write_estimate_error_report(ERROR_REPORT_PATH, &estimate)
                            {
                                status = format!(
                                    "{} (failed to write {}: {})",
                                    status, ERROR_REPORT_PATH, report_error
                                );
                            }

                            *status_arc_inner.lock().unwrap() = status;
                        }
                        Err(e) => {
                            *cost_arc_inner.lock().unwrap() = format!("Estimate error: {e}");
                            *status_arc_inner.lock().unwrap() = "Cost estimate failed".to_string();
                        }
                    }
                });
            }

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

            ui.separator();
            ui.label("Estimated Cost (USD):");
            {
                if ui.button("Copy Estimate").clicked() {
                    let estimate_text = self.cost_estimate.lock().unwrap().clone();
                    ui.ctx().copy_text(estimate_text);
                    *self.task_status.lock().unwrap() = "Estimate copied to clipboard".to_string();
                }

                let estimate_text = self.cost_estimate.lock().unwrap().clone();
                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::Label::new(egui::RichText::new(estimate_text).monospace())
                                .selectable(true)
                                .wrap(),
                        );
                    });
            }
            ui.small("Estimate only. This does not start a download.");

            ui.label(&*self.task_status.lock().unwrap());
        });
    }
}
