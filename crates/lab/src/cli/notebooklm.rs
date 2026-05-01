//! `lab notebooklm` — thin CLI shim for the NotebookLM service.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::cli::helpers::{action_parser, print_dry_run, run_confirmable_action_command};
use crate::dispatch::notebooklm::ACTIONS;
use crate::output::OutputFormat;

#[derive(Debug, Args)]
pub struct NotebooklmArgs {
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

pub async fn run(args: NotebooklmArgs, format: OutputFormat) -> Result<ExitCode> {
    let params = args
        .params
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    if args.dry_run {
        print_dry_run("notebooklm", &args.action, &params);
        return Ok(ExitCode::SUCCESS);
    }
    run_confirmable_action_command(
        "notebooklm",
        ACTIONS,
        args.action,
        params,
        args.yes,
        format,
        |action, params| async move {
            crate::dispatch::notebooklm::dispatch(&action, params).await
        },
    )
    .await
}
