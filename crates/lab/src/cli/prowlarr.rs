//! `lab prowlarr` — CLI stub.
//!
//! Thin shim: parse → dispatch layer → format. See `radarr.rs` for the typed reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::cli::helpers::run_action_command;
use crate::output::OutputFormat;

/// `lab prowlarr` arguments.
#[derive(Debug, Args)]
pub struct ProwlarrArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
    /// Action-specific parameters as JSON.
    #[arg(long)]
    pub params: Option<String>,
}

/// Run the `lab prowlarr` subcommand.
///
/// # Errors
/// Returns an error if dispatch fails.
pub async fn run(args: ProwlarrArgs, format: OutputFormat) -> Result<ExitCode> {
    let action = args.action.unwrap_or_else(|| "help".to_string());
    let params = args
        .params
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    run_action_command(
        "prowlarr",
        action,
        params,
        format,
        |action, params| async move {
            crate::dispatch::prowlarr::dispatch(&action, params).await
        },
    )
    .await
}
