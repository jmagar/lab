//! `lab sonarr` — CLI thin shim over the shared dispatch layer.
//!
//! Thin shim: parse → dispatch → format.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::cli::helpers::run_action_command;
use crate::output::OutputFormat;

/// `lab sonarr` arguments.
#[derive(Debug, Args)]
pub struct SonarrArgs {
    /// Action to run (e.g. help, series.list, episode.list).
    pub action: Option<String>,
    /// Action-specific parameters as JSON.
    #[arg(long)]
    pub params: Option<String>,
}

/// Run the `lab sonarr` subcommand.
///
/// # Errors
/// Returns an error if dispatch fails.
pub async fn run(args: SonarrArgs, format: OutputFormat) -> Result<ExitCode> {
    let action = args.action.unwrap_or_else(|| "help".to_string());
    let params = args
        .params
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    run_action_command(
        "sonarr",
        action,
        params,
        format,
        |action, params| async move { crate::dispatch::sonarr::dispatch(&action, params).await },
    )
    .await
}
