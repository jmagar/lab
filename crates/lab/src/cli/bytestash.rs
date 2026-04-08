//! `lab bytestash` — CLI stub (not yet implemented).
//!
//! Replace this stub once `bytestash` SDK client is complete.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::output::OutputFormat;

/// `lab bytestash` arguments.
#[derive(Debug, Args)]
pub struct BytestashArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
}

/// Run the `lab bytestash` subcommand stub.
///
/// # Errors
/// Always returns a not-yet-implemented message.
pub async fn run(_args: BytestashArgs, _format: OutputFormat) -> Result<ExitCode> {
    anyhow::bail!("bytestash is not yet implemented — run `lab help` for available services")
}
