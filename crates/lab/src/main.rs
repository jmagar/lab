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
mod tui;

use std::process::ExitCode;

use clap::Parser;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::cli::Cli;

fn init_tracing() {
    let filter = EnvFilter::try_from_env("LAB_LOG")
        .unwrap_or_else(|_| EnvFilter::new("lab=info,lab_apis=warn"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false))
        .init();
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
