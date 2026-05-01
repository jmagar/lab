use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::cli::helpers::run_confirmable_action_command;
use crate::cli::params::parse_kv_params;
use crate::dispatch::loggifly::ACTIONS;
use crate::output::OutputFormat;

// TODO: Replace this generic action+params interface with typed subcommands that
// mirror the dispatch catalog. See crates/lab/src/cli/radarr.rs for an example.

/// `lab loggifly` arguments.
#[derive(Debug, Args)]
pub struct LoggiflyArgs {
    /// Action to run, e.g. `help` or `schema`.
    pub action: String,

    /// Optional `key=value` params for the action.
    #[arg(value_name = "KEY=VALUE", trailing_var_arg = true)]
    pub params: Vec<String>,

    /// Skip confirmation for destructive actions.
    #[arg(short = 'y', long, alias = "no-confirm")]
    pub yes: bool,
}

/// Run the `lab loggifly` subcommand.
pub async fn run(args: LoggiflyArgs, format: OutputFormat) -> Result<ExitCode> {
    let params = parse_kv_params(args.params)?;
    run_confirmable_action_command(
        "loggifly",
        ACTIONS,
        args.action,
        params,
        args.yes,
        format,
        |action, params| async move { crate::dispatch::loggifly::dispatch(&action, params).await },
    )
    .await
}
