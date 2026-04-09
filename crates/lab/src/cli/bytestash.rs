//! `lab bytestash` — CLI shim for the `ByteStash` service.
//!
//! Thin shim: parse action + key/value params, call the shared dispatcher,
//! and format the result. This mirrors the MCP action surface directly.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::output::{OutputFormat, print};
use crate::cli::params::parse_kv_params;

/// `lab bytestash` arguments.
#[derive(Debug, Args)]
pub struct BytestashArgs {
    /// Action to run, e.g. `help`, `snippets.list`, `categories.list`.
    pub action: String,

    /// Optional `key=value` params for the action.
    #[arg(value_name = "KEY=VALUE", trailing_var_arg = true)]
    pub params: Vec<String>,
}

/// Run the `lab bytestash` subcommand.
///
/// # Errors
/// Returns an error if the client is not configured or the API call fails.
pub async fn run(args: BytestashArgs, format: OutputFormat) -> Result<ExitCode> {
    let params = parse_kv_params(args.params)?;
    let start = std::time::Instant::now();
    let result = crate::services::bytestash::dispatch(&args.action, params).await;
    let elapsed_ms = start.elapsed().as_millis();
    match &result {
        Ok(_) => tracing::info!(
            surface = "cli",
            service = "bytestash",
            action = args.action,
            elapsed_ms,
            "dispatch ok"
        ),
        Err(e) => tracing::warn!(
            surface = "cli",
            service = "bytestash",
            action = args.action,
            elapsed_ms,
            kind = e.kind(),
            "dispatch error"
        ),
    }
    let value = result.map_err(|te| {
        anyhow::anyhow!(
            "{}",
            serde_json::to_string(&te).unwrap_or_else(|_| format!("{te:?}"))
        )
    })?;
    print(&value, format)?;
    Ok(ExitCode::SUCCESS)
}


