//! `lab arcane` — CLI shim for the Arcane Docker management service.
//!
//! Thin shim: parse action + key/value params, call the shared dispatcher,
//! and format the result.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::cli::helpers::{action_parser, run_confirmable_action_command};
use crate::cli::params::parse_kv_params;
use crate::dispatch::arcane::ACTIONS;
use crate::output::OutputFormat;

/// `lab arcane` arguments.
#[derive(Debug, Args)]
pub struct ArcaneArgs {
    /// Action to run, e.g. `help`, `system.health`, `container.list`.
    #[arg(default_value = "help", value_parser = action_parser(ACTIONS))]
    pub action: String,

    /// Optional `key=value` params for the action.
    #[arg(value_name = "KEY=VALUE", trailing_var_arg = true)]
    pub params: Vec<String>,

    /// Skip confirmation for destructive actions.
    #[arg(short = 'y', long, alias = "no-confirm")]
    pub yes: bool,

    /// Print what would be done without executing.
    #[arg(long)]
    pub dry_run: bool,
}

/// Run the `lab arcane` subcommand.
///
/// # Errors
/// Returns an error if the client is not configured or the API call fails.
#[allow(clippy::print_stdout)]
pub async fn run(args: ArcaneArgs, format: OutputFormat) -> Result<ExitCode> {
    let action = args.action;
    let params = parse_kv_params(args.params)?;
    if args.dry_run {
        if !ACTIONS.iter().any(|a| a.name == action) {
            anyhow::bail!(
                "unknown arcane action `{action}`; valid: {}",
                ACTIONS
                    .iter()
                    .map(|a| a.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        println!(
            "[dry-run] would dispatch arcane action `{}` with params: {}",
            action,
            serde_json::to_string(&params).unwrap_or_else(|_| "{}".to_string())
        );
        return Ok(ExitCode::SUCCESS);
    }
    run_confirmable_action_command(
        "arcane",
        ACTIONS,
        action,
        params,
        args.yes,
        format,
        |action, params| async move { crate::dispatch::arcane::dispatch(&action, params).await },
    )
    .await
}
