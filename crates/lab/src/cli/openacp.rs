//! `lab openacp` - thin CLI shim for the upstream OpenACP daemon.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::cli::helpers::{action_parser, print_dry_run, run_confirmable_action_command};
use crate::dispatch::openacp::ACTIONS;
use crate::output::OutputFormat;

/// `lab openacp` arguments.
#[derive(Debug, Args)]
pub struct OpenAcpArgs {
    /// Action to run, for example `system.health`.
    #[arg(default_value = "help", value_parser = action_parser(ACTIONS))]
    pub action: String,
    /// Action-specific parameters as JSON.
    #[arg(long)]
    pub params: Option<String>,
    /// Select a named OpenACP instance.
    #[arg(long)]
    pub instance: Option<String>,
    /// Accepted for consistency; OpenACP actions are not destructive-gated.
    #[arg(short = 'y', long, alias = "no-confirm")]
    pub yes: bool,
    /// Print what would be done without executing.
    #[arg(long)]
    pub dry_run: bool,
}

/// Run the `lab openacp` subcommand.
pub async fn run(args: OpenAcpArgs, format: OutputFormat) -> Result<ExitCode> {
    let mut params = args
        .params
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    if let Some(instance) = args.instance
        && let serde_json::Value::Object(ref mut map) = params
    {
        map.insert("instance".to_string(), serde_json::Value::String(instance));
    }
    if args.dry_run {
        print_dry_run("openacp", &args.action, &params);
        return Ok(ExitCode::SUCCESS);
    }
    run_confirmable_action_command(
        "openacp",
        ACTIONS,
        args.action,
        params,
        args.yes,
        format,
        |action, params| async move { crate::dispatch::openacp::dispatch(&action, params).await },
    )
    .await
}
