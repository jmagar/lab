//! `lab` binary entry point.
//!
//! Initializes tracing, loads config, parses clap args, and dispatches
//! to the appropriate subcommand handler. All subsystems are sibling
//! modules declared here.

#![allow(clippy::multiple_crate_versions)]
#![allow(unreachable_pub)] // binary crate — `pub` items are crate-internal by design

mod api;
mod audit;
mod catalog;
mod cli;
mod config;
mod device;
mod dispatch;
mod mcp;
mod oauth;
mod output;
mod registry;
mod scaffold;
#[cfg(test)]
mod test_support;
mod tui;

use std::process::ExitCode;

use clap::Parser;
use is_terminal::IsTerminal;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::cli::Cli;
use crate::dispatch::logs::ingest::LogIngestLayer;

/// Initialize tracing.
///
/// Accepts config.toml log preferences; env vars `LAB_LOG` / `LAB_LOG_FORMAT`
/// override them when set.
fn init_tracing(log: &config::LogPreferences) {
    // Env var wins → config.toml → default.
    let filter = EnvFilter::try_from_env("LAB_LOG").unwrap_or_else(|_| {
        let directive = log.filter.as_deref().unwrap_or("lab=info,lab_apis=warn");
        EnvFilter::new(directive)
    });

    let use_json = match std::env::var("LAB_LOG_FORMAT").ok() {
        Some(v) => v.eq_ignore_ascii_case("json"),
        None => log
            .format
            .as_deref()
            .is_some_and(|f| f.eq_ignore_ascii_case("json")),
    };

    if use_json {
        tracing_subscriber::registry()
            .with(filter)
            .with(LogIngestLayer)
            .with(fmt::layer().json().with_writer(std::io::stderr))
            .init();
    } else {
        // Enable ANSI colors only when stderr is an interactive terminal.
        let ansi = std::io::stderr().is_terminal();
        tracing_subscriber::registry()
            .with(filter)
            .with(LogIngestLayer)
            .with(
                fmt::layer()
                    .with_target(false)
                    .with_ansi(ansi)
                    .with_writer(std::io::stderr),
            )
            .init();
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    // 1. Load config.toml first (lightweight, no tracing needed).
    //    eprintln is intentional — tracing isn't initialized yet.
    let config = match config::load_toml(&config::toml_candidates()) {
        Ok(cfg) => cfg,
        Err(err) => {
            #[allow(clippy::print_stderr)]
            {
                eprintln!("config.toml parse error: {err:#}");
            }
            return ExitCode::from(2);
        }
    };

    // 2. Init tracing using config.toml [log] preferences (env vars override).
    init_tracing(&config.log);

    // 3. Load .env files (secrets + URL env vars).
    if let Err(err) = config::load_dotenv() {
        tracing::error!("dotenv load error: {err:#}");
        return ExitCode::from(2);
    }

    let cli = Cli::parse();

    match cli::dispatch(cli, config).await {
        Ok(code) => code,
        Err(err) => {
            tracing::error!("{err:#}");
            ExitCode::from(1)
        }
    }
}
