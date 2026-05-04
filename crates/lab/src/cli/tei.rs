//! `labby tei` — thin CLI shim for the TEI (Text Embeddings Inference) service.
//!
//! Thin shim: parse → shared dispatch layer → format.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::cli::helpers::{action_parser, run_action_command};
use crate::dispatch::tei::ACTIONS;
use crate::output::OutputFormat;

/// `labby tei` arguments.
#[derive(Debug, Args)]
pub struct TeiArgs {
    /// Action to run (e.g. help).
    #[arg(default_value = "help", value_parser = action_parser(ACTIONS))]
    pub action: String,
    /// Action-specific parameters as JSON.
    #[arg(long)]
    pub params: Option<String>,
}

/// Run the `labby tei` subcommand.
///
/// # Errors
/// Returns an error if dispatch fails.
pub async fn run(args: TeiArgs, format: OutputFormat) -> Result<ExitCode> {
    let params = args
        .params
        .as_deref()
        .map(serde_json::from_str)
        .transpose()?
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    run_action_command(
        "tei",
        args.action,
        params,
        format,
        |action, params| async move { crate::dispatch::tei::dispatch(&action, params).await },
    )
    .await
}
