//! `lab mcpregistry` — CLI shim for the MCP Registry service.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::cli::helpers::run_confirmable_action_command;
use crate::cli::params::parse_kv_params;
use crate::dispatch::mcpregistry::ACTIONS;
use crate::output::OutputFormat;

/// `lab mcpregistry` arguments.
#[derive(Debug, Args)]
pub struct McpregistryArgs {
    /// Action to run, e.g. `help`, `server.list`, `server.get`.
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

/// Run the `lab mcpregistry` subcommand.
///
/// # Errors
/// Returns an error if the client cannot be initialized or the API call fails.
#[allow(clippy::print_stdout)]
pub async fn run(args: McpregistryArgs, format: OutputFormat) -> Result<ExitCode> {
    let params = parse_kv_params(args.params)?;
    if args.dry_run {
        println!(
            "[dry-run] would dispatch mcpregistry action `{}` with params: {}",
            args.action,
            serde_json::to_string(&params).unwrap_or_else(|_| "{}".to_string())
        );
        return Ok(ExitCode::SUCCESS);
    }
    run_confirmable_action_command(
        "mcpregistry",
        ACTIONS,
        args.action,
        params,
        args.yes,
        format,
        |action, params| async move { crate::dispatch::mcpregistry::dispatch(&action, params).await },
    )
    .await
}
