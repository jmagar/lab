//! `lab sabnzbd` — CLI shim for the `SABnzbd` service.
//!
//! Thin shim: parse action + JSON params, call the shared dispatcher,
//! and format the result.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::cli::helpers::run_action_command;
use crate::output::OutputFormat;

/// `lab sabnzbd` arguments.
#[derive(Debug, Args)]
pub struct SabnzbdArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
    /// Action-specific parameters as JSON.
    #[arg(long)]
    pub params: Option<String>,
}

/// Run the `lab sabnzbd` subcommand.
///
/// # Errors
/// Returns an error if dispatch fails.
pub async fn run(args: SabnzbdArgs, format: OutputFormat) -> Result<ExitCode> {
    let action = args.action.unwrap_or_else(|| "help".to_string());
    let params = args
        .params
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?
        .unwrap_or(serde_json::Value::Null);

    run_action_command(
        "sabnzbd",
        action,
        params,
        format,
        |action, params| async move { crate::dispatch::sabnzbd::dispatch(&action, params).await },
    )
    .await
}
