use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::cli::helpers::run_confirmable_action_command;
use crate::cli::params::parse_kv_params;
use crate::dispatch::dozzle::ACTIONS;
use crate::output::OutputFormat;

// TODO: Replace this generic action+params interface with typed subcommands that
// mirror the dispatch catalog. See crates/lab/src/cli/radarr.rs for an example.

/// `lab dozzle` arguments.
#[derive(Debug, Args)]
pub struct DozzleArgs {
    /// Action to run, e.g. `help` or `schema`.
    pub action: String,

    /// Optional `key=value` params for the action.
    #[arg(value_name = "KEY=VALUE", trailing_var_arg = true)]
    pub params: Vec<String>,

    /// Skip confirmation for destructive actions.
    #[arg(short = 'y', long, alias = "no-confirm")]
    pub yes: bool,
}

/// Run the `lab dozzle` subcommand.
pub async fn run(args: DozzleArgs, format: OutputFormat) -> Result<ExitCode> {
    let params = parse_kv_params(args.params)?;
    run_confirmable_action_command(
        "dozzle",
        ACTIONS,
        args.action,
        params,
        args.yes,
        format,
        |action, params| async move { crate::dispatch::dozzle::dispatch(&action, params).await },
    )
    .await
}
