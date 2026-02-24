use anyhow::{anyhow, Result};
use clap::Parser;
use eframe::egui;

mod cli;
mod client;
mod commands;
mod downloader;
mod types;
mod gui;
mod custom_datepicker;

use crate::cli::{Cli, Commands};
use crate::commands::get_quote::estimate_quote_cost;

fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        return run_cli_command(command);
    }

    run_gui()
}

fn run_gui() -> Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Databento Toolkit",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_theme(egui::Theme::Dark);
            Ok(Box::new(gui::AppState::default()))}),
    )
    .map_err(|e| anyhow!("GUI startup failed: {e}"))?;
    Ok(())
}

fn run_cli_command(command: Commands) -> Result<()> {
    match command {
        Commands::Quote(args) => {
            let request = args.into_request()?;

            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;

            let cost = runtime.block_on(estimate_quote_cost(&request))?;
            println!(
                "Cost estimate for {} from {} to {}: ${:.4}",
                request.symbol, request.start, request.end, cost
            );
        }
    }

    Ok(())
}
