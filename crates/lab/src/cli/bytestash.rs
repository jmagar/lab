//! `lab bytestash` — CLI shim for the `ByteStash` service.
//!
//! Thin shim: parse action + key/value params, call the shared dispatcher,
//! and format the result. This mirrors the MCP action surface directly.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::cli::helpers::run_confirmable_action_command;
use crate::cli::params::parse_kv_params;
use crate::dispatch::bytestash::ACTIONS;
use crate::output::OutputFormat;

/// `lab bytestash` arguments.
#[derive(Debug, Args)]
pub struct BytestashArgs {
    /// Action to run, e.g. `help`, `snippets.list`, `categories.list`.
    pub action: String,

    /// Optional `key=value` params for the action.
    #[arg(value_name = "KEY=VALUE", trailing_var_arg = true)]
    pub params: Vec<String>,

    /// Skip confirmation for destructive actions.
    #[arg(short = 'y', long, alias = "no-confirm")]
    pub yes: bool,
}

/// Run the `lab bytestash` subcommand.
///
/// # Errors
/// Returns an error if the client is not configured or the API call fails.
pub async fn run(args: BytestashArgs, format: OutputFormat) -> Result<ExitCode> {
    let params = parse_kv_params(args.params)?;
    run_confirmable_action_command(
        "bytestash",
        ACTIONS,
        args.action,
        params,
        args.yes,
        format,
        |action, params| async move { crate::dispatch::bytestash::dispatch(&action, params).await },
    )
    .await
}
