//! `lab unraid` — CLI stub (not yet implemented).
//!
//! Thin shim: parse → MCP dispatch → format. Replace once SDK client is complete.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::cli::helpers::run_action_command;
use crate::output::OutputFormat;

/// `lab unraid` arguments.
#[derive(Debug, Args)]
pub struct UnraidArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
    /// Action-specific parameters as JSON.
    #[arg(long)]
    pub params: Option<String>,
}

/// Run the `lab unraid` subcommand.
///
/// # Errors
/// Returns an error if dispatch fails.
pub async fn run(args: UnraidArgs, format: OutputFormat) -> Result<ExitCode> {
    let action = args.action.unwrap_or_else(|| "help".to_string());
    let params = args
        .params
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    run_action_command(
        "unraid",
        action,
        params,
        format,
        |action, params| async move {
            crate::dispatch::unraid::dispatch(&action, params).await
        },
    )
    .await
}
