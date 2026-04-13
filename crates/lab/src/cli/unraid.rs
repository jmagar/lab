//! `lab unraid` — CLI stub (not yet implemented).
//!
//! Thin shim: parse → MCP dispatch → format. Replace once SDK client is complete.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::cli::helpers::run_confirmable_action_command;
use crate::dispatch::unraid::ACTIONS;
use crate::output::OutputFormat;

/// `lab unraid` arguments.
#[derive(Debug, Args)]
pub struct UnraidArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
    /// Optional named instance label.
    #[arg(long)]
    pub instance: Option<String>,
    /// Action-specific parameters as JSON.
    #[arg(long)]
    pub params: Option<String>,
    /// Skip confirmation prompt for destructive actions (docker.start/stop/restart).
    #[arg(short = 'y', long, alias = "no-confirm")]
    pub yes: bool,
}

/// Run the `lab unraid` subcommand.
///
/// # Errors
/// Returns an error if dispatch fails.
pub async fn run(args: UnraidArgs, format: OutputFormat) -> Result<ExitCode> {
    let action = args.action.unwrap_or_else(|| "help".to_string());
    let mut params = args
        .params
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    if let Some(instance) = args.instance {
        if let serde_json::Value::Object(ref mut map) = params {
            map.insert("instance".to_string(), serde_json::Value::String(instance));
        } else {
            anyhow::bail!("--instance requires --params to be a JSON object");
        }
    }
    run_confirmable_action_command(
        "unraid",
        ACTIONS,
        action,
        params,
        args.yes,
        format,
        |action, params| async move { crate::dispatch::unraid::dispatch(&action, params).await },
    )
    .await
}
