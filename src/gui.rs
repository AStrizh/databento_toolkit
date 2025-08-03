use crate::commands::download::download_history;
use crate::commands::get_quote::get_quote;
use crate::downloader::decode::decode_all_in_dir;
use crate::types::JsonOhlcv;
use anyhow::Result;
use eframe::{egui, App};
use std::sync::{Arc, Mutex};
use time::{Date};



// ───── GUI App State ─────
pub(crate) struct AppState {
    start_date: String,
    end_date: String,
    symbols_input: String,
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
            start_date: String::new(),
            end_date: String::new(),
            symbols_input: String::new(),
            task_status: Arc::new(Mutex::new(String::new())),
            runtime,
        }
    }
}

impl App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Databento Toolkit GUI");

            ui.horizontal(|ui| {
                ui.label("Start Date (YYYY-MM-DD):");
                ui.text_edit_singleline(&mut self.start_date);
            });

            ui.horizontal(|ui| {
                ui.label("End Date (YYYY-MM-DD):");
                ui.text_edit_singleline(&mut self.end_date);
            });

            ui.horizontal(|ui| {
                ui.label("Symbols (comma-separated):");
                ui.text_edit_singleline(&mut self.symbols_input);
            });

            let status_arc = self.task_status.clone();

            if ui.button("Download History").clicked() {
                let start = Date::parse(&self.start_date, &time::macros::format_description!("[year]-[month]-[day]"));
                let end = Date::parse(&self.end_date, &time::macros::format_description!("[year]-[month]-[day]"));

                match (start, end) {
                    (Ok(start_date), Ok(end_date)) => {
                        *status_arc.lock().unwrap() = "Downloading...".to_string();

                        let symbols: Vec<String> = self
                            .symbols_input
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect();

                        let status_arc_inner = status_arc.clone();
                        self.runtime.spawn(async move {
                            let result = download_history(
                                start_date,
                                end_date,
                                &symbols.iter().map(AsRef::as_ref).collect::<Vec<_>>(),
                                "Hist_Fut_Data",
                            ).await;

                            let mut status = status_arc_inner.lock().unwrap();
                            *status = match result {
                                Ok(_) => "Download complete".to_string(),
                                Err(e) => format!("Error: {}", e),
                            };
                        });
                    }
                    _ => {
                        *status_arc.lock().unwrap() = "Invalid date format. Use YYYY-MM-DD".to_string();
                    }
                }
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