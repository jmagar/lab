//! `lab` binary entry point.
//!
//! Initializes tracing, loads config, parses clap args, and dispatches
//! to the appropriate subcommand handler. All subsystems are sibling
//! modules declared here.

#![allow(clippy::multiple_crate_versions)]
#![allow(unreachable_pub)] // binary crate — `pub` items are crate-internal by design

mod acp;
mod api;
mod audit;
mod catalog;
mod cli;
mod config;
mod dispatch;
mod log_fmt;
mod mcp;
mod node;
mod oauth;
mod output;
mod process;
mod registry;
mod scaffold;
#[cfg(test)]
mod test_support;
mod tui;

use std::process::ExitCode;

use clap::Parser;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, filter::filter_fn, fmt, prelude::*};

use crate::cli::Cli;
use crate::dispatch::logs::ingest::LogIngestLayer;
use crate::log_fmt::formatter::PremiumEventFormatter;
use crate::output::{ColorPolicy, RenderEnv, human_output_styling_enabled};

fn human_console_target_enabled(target: &str) -> bool {
    target == "lab"
        || target.starts_with("lab::")
        || target == "lab_apis"
        || target.starts_with("lab_apis::")
        || target == "lab_auth"
        || target.starts_with("lab_auth::")
}

/// Initialize tracing.
///
/// Accepts config.toml log preferences; env vars `LAB_LOG` / `LAB_LOG_FORMAT`
/// override them when set.
fn init_tracing(log: &config::LogPreferences, color_policy: ColorPolicy, filter_override: Option<&str>) -> tracing_appender::non_blocking::WorkerGuard {
    // Priority: explicit CLI override > LAB_LOG env var > config.toml > default.
    let filter = if let Some(directive) = filter_override {
        EnvFilter::new(directive)
    } else {
        EnvFilter::try_from_env("LAB_LOG").unwrap_or_else(|_| {
            let directive = log
                .filter
                .as_deref()
                .unwrap_or("lab=info,lab_apis=warn,rmcp=warn");
            EnvFilter::new(directive)
        })
    };

    // ── Rolling file appender (survives OOM — guard must live as long as main) ──
    let log_dir = std::env::var("LAB_LOG_DIR")
        .unwrap_or_else(|_| format!(
            "{}/.local/share/lab/logs",
            std::env::var("HOME").unwrap_or_default()
        ));
    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("lab")
        .filename_suffix("log")
        .max_log_files(7)
        .build(&log_dir)
        .expect("failed to create lab log file appender");

    let (non_blocking_file, _log_guard) = tracing_appender::non_blocking(file_appender);

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
            .with(fmt::layer().json().with_writer(std::io::stderr))           // console
            .with(fmt::layer().json().with_writer(non_blocking_file))         // file
            .init();
    } else {
        let fmt_layer = fmt::layer()
            .with_ansi(human_output_styling_enabled(
                color_policy,
                RenderEnv::stderr(),
            ))
            .with_target(false)
            .event_format(PremiumEventFormatter)
            .with_writer(std::io::stderr)
            .with_filter(filter_fn(|metadata| {
                human_console_target_enabled(metadata.target())
            }));
        tracing_subscriber::registry()
            .with(filter)
            .with(LogIngestLayer)
            .with(fmt_layer)                                                   // console (pretty)
            .with(fmt::layer().json().with_writer(non_blocking_file))         // file (JSON)
            .init();
    }

    _log_guard
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

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

    // 2. Init tracing. If `lab serve --log-level <level>` was given, pass it
    //    directly to avoid mutating the environment (crate forbids unsafe_code).
    let log_filter_override: Option<String> = if let crate::cli::Command::Serve(ref args) = cli.command {
        args.log_level.as_ref().map(|level| format!("lab={level},warn"))
    } else {
        None
    };
    // _log_guard MUST live for the entire process — dropping it stops file logging.
    let _log_guard = init_tracing(&config.log, cli.color, log_filter_override.as_deref());

    // 3. Load .env files (secrets + URL env vars).
    if let Err(err) = config::load_dotenv() {
        tracing::error!("dotenv load error: {err:#}");
        return ExitCode::from(2);
    }

    match cli::dispatch(cli, config).await {
        Ok(code) => code,
        Err(err) => {
            tracing::error!("{err:#}");
            ExitCode::from(1)
        }
    }
}
