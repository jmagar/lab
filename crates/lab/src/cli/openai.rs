//! `labby openai` — thin CLI shim for the OpenAI service.
//!
//! Thin shim: parse → shared dispatch layer → format.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::cli::helpers::{action_parser, run_action_command};
use crate::dispatch::openai::ACTIONS;
use crate::output::OutputFormat;

/// `labby openai` arguments.
#[derive(Debug, Args)]
pub struct OpenaiArgs {
    /// Action to run (e.g. help).
    #[arg(default_value = "help", value_parser = action_parser(ACTIONS))]
    pub action: String,
    /// Action-specific parameters as JSON.
    #[arg(long)]
    pub params: Option<String>,
}

/// Run the `labby openai` subcommand.
///
/// # Errors
/// Returns an error if dispatch fails.
pub async fn run(args: OpenaiArgs, format: OutputFormat) -> Result<ExitCode> {
    let params = args
        .params
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    run_action_command(
        "openai",
        args.action,
        params,
        format,
        |action, params| async move { crate::dispatch::openai::dispatch(&action, params).await },
    )
    .await
}
