//! `lab arcane` — CLI stub (not yet implemented).
//!
//! Thin shim: parse → MCP dispatch → format. Replace once SDK client is complete.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::output::{OutputFormat, print};

/// `lab arcane` arguments.
#[derive(Debug, Args)]
pub struct ArcaneArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
    /// Action-specific parameters as JSON.
    #[arg(long)]
    pub params: Option<String>,
}

/// Run the `lab arcane` subcommand.
///
/// # Errors
/// Returns an error if dispatch fails.
pub async fn run(args: ArcaneArgs, format: OutputFormat) -> Result<ExitCode> {
    let action = args.action.as_deref().unwrap_or("help");
    let params = args
        .params
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?
        .unwrap_or(serde_json::Value::Null);
    let result = crate::mcp::services::arcane::dispatch(action, params)
        .await
        .map_err(|te| {
            anyhow::anyhow!(
                "{}",
                serde_json::to_string(&te).unwrap_or_else(|_| format!("{te:?}"))
            )
        })?;
    print(&result, format)?;
    Ok(ExitCode::SUCCESS)
}
