//! `lab unraid` — thin CLI shim for the Unraid service.
//!
//! Thin shim: parse → shared dispatch layer → format.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::cli::helpers::{action_parser, print_dry_run, run_confirmable_action_command};
use crate::dispatch::unraid::ACTIONS;
use crate::output::OutputFormat;

/// `lab unraid` arguments.
#[derive(Debug, Args)]
pub struct UnraidArgs {
    /// Action to run (e.g. help).
    #[arg(default_value = "help", value_parser = action_parser(ACTIONS))]
    pub action: String,
    /// Optional named instance label.
    #[arg(long)]
    pub instance: Option<String>,
    /// Action-specific parameters as JSON.
    #[arg(long)]
    pub params: Option<String>,
    /// Skip confirmation prompt for destructive actions (docker.start/stop/restart).
    #[arg(short = 'y', long, alias = "no-confirm")]
    pub yes: bool,
    /// Print what would be done without executing.
    #[arg(long)]
    pub dry_run: bool,
}

/// Run the `lab unraid` subcommand.
///
/// # Errors
/// Returns an error if dispatch fails.
#[allow(clippy::print_stdout)]
pub async fn run(args: UnraidArgs, format: OutputFormat) -> Result<ExitCode> {
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
    if args.dry_run {
        print_dry_run("unraid", &args.action, &params);
        return Ok(ExitCode::SUCCESS);
    }
    run_confirmable_action_command(
        "unraid",
        ACTIONS,
        args.action,
        params,
        args.yes,
        format,
        |action, params| async move { crate::dispatch::unraid::dispatch(&action, params).await },
    )
    .await
}
