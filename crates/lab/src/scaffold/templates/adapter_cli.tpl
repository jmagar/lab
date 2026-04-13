use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::cli::helpers::run_confirmable_action_command;
use crate::cli::params::parse_kv_params;
use crate::dispatch::{{service}}::ACTIONS;
use crate::output::OutputFormat;

/// `lab {{service}}` arguments.
#[derive(Debug, Args)]
pub struct {{Service}}Args {
    /// Action to run, e.g. `help` or `schema`.
    pub action: String,

    /// Optional `key=value` params for the action.
    #[arg(value_name = "KEY=VALUE", trailing_var_arg = true)]
    pub params: Vec<String>,

    /// Skip confirmation for destructive actions.
    #[arg(short = 'y', long, alias = "no-confirm")]
    pub yes: bool,
}

/// Run the `lab {{service}}` subcommand.
pub async fn run(args: {{Service}}Args, format: OutputFormat) -> Result<ExitCode> {
    let params = parse_kv_params(args.params)?;
    run_confirmable_action_command(
        "{{service}}",
        ACTIONS,
        args.action,
        params,
        args.yes,
        format,
        |action, params| async move { crate::dispatch::{{service}}::dispatch(&action, params).await },
    )
    .await
}
