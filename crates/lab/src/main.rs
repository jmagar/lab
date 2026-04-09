//! `lab` binary entry point.
//!
//! Initializes tracing, loads config, parses clap args, and dispatches
//! to the appropriate subcommand handler. All subsystems are sibling
//! modules declared here.

#![allow(clippy::multiple_crate_versions)]
#![allow(unreachable_pub)] // binary crate — `pub` items are crate-internal by design
#![allow(dead_code)] // skeleton: many surfaces are wired but not yet consumed

mod api;
mod catalog;
mod cli;
mod config;
mod mcp;
mod output;
mod services;
mod tui;

use std::process::ExitCode;

use clap::Parser;
use is_terminal::IsTerminal;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::cli::Cli;

/// Initialize tracing.
///
/// - `LAB_LOG` — filter directive (default: `lab=info,lab_apis=warn`).
/// - `LAB_LOG_FORMAT=json` — emit newline-delimited JSON instead of human-readable text.
fn init_tracing() {
    let filter = EnvFilter::try_from_env("LAB_LOG")
        .unwrap_or_else(|_| EnvFilter::new("lab=info,lab_apis=warn"));

    let use_json = std::env::var("LAB_LOG_FORMAT")
        .map(|v| v.eq_ignore_ascii_case("json"))
        .unwrap_or(false);

    if use_json {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().json())
            .init();
    } else {
        // Enable ANSI colors only when stderr is an interactive terminal.
        let ansi = std::io::stderr().is_terminal();
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_target(false).with_ansi(ansi))
            .init();
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> ExitCode {
    init_tracing();

    if let Err(err) = config::load() {
        tracing::warn!("config load warning: {err:#}");
    }

    let cli = Cli::parse();

    match cli::dispatch(cli).await {
        Ok(code) => code,
        Err(err) => {
            tracing::error!("{err:#}");
            ExitCode::from(1)
        }
    }
}
