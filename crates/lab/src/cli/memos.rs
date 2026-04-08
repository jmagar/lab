//! `lab memos` — CLI stub (not yet implemented).
//!
//! Replace this stub once `memos` SDK client is complete.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::output::OutputFormat;

/// `lab memos` arguments.
#[derive(Debug, Args)]
pub struct MemosArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
}

/// Run the `lab memos` subcommand stub.
///
/// # Errors
/// Always returns a not-yet-implemented message.
pub async fn run(_args: MemosArgs, _format: OutputFormat) -> Result<ExitCode> {
    anyhow::bail!("memos is not yet implemented — run `lab help` for available services")
}
