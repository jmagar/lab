//! `lab apprise` — CLI stub (not yet implemented).
//!
//! Replace this stub once `apprise` SDK client is complete.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::output::OutputFormat;

/// `lab apprise` arguments.
#[derive(Debug, Args)]
pub struct AppriseArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
}

/// Run the `lab apprise` subcommand stub.
///
/// # Errors
/// Always returns a not-yet-implemented message.
pub async fn run(_args: AppriseArgs, _format: OutputFormat) -> Result<ExitCode> {
    anyhow::bail!("apprise is not yet implemented — run `lab help` for available services")
}
