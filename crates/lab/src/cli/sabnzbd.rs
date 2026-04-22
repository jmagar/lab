//! `lab sabnzbd` — CLI shim for the SABnzbd service.
//!
//! Thin shim: parse action + JSON params, call the shared dispatcher,
//! and format the result.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::cli::helpers::{action_parser, print_dry_run, run_confirmable_action_command};
use crate::dispatch::sabnzbd::ACTIONS;
use crate::output::OutputFormat;

/// `lab sabnzbd` arguments.
#[derive(Debug, Args)]
pub struct SabnzbdArgs {
    /// Action to run (e.g. help).
    #[arg(default_value = "help", value_parser = action_parser(ACTIONS))]
    pub action: String,
    /// Action-specific parameters as JSON.
    #[arg(long)]
    pub params: Option<String>,
    /// Skip confirmation for destructive actions.
    #[arg(short = 'y', long, alias = "no-confirm")]
    pub yes: bool,
    /// Print what would be done without executing.
    #[arg(long)]
    pub dry_run: bool,
}

/// Run the `lab sabnzbd` subcommand.
///
/// # Errors
/// Returns an error if dispatch fails.
pub async fn run(args: SabnzbdArgs, format: OutputFormat) -> Result<ExitCode> {
    let params = args
        .params
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    if args.dry_run {
        print_dry_run("sabnzbd", &args.action, &params);
        return Ok(ExitCode::SUCCESS);
    }
    run_confirmable_action_command(
        "sabnzbd",
        ACTIONS,
        args.action,
        params,
        args.yes,
        format,
        |action, params| async move { crate::dispatch::sabnzbd::dispatch(&action, params).await },
    )
    .await
}
