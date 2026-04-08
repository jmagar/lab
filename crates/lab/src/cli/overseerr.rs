//! `lab overseerr` — CLI stub (not yet implemented).
//!
//! Replace this stub once `overseerr` SDK client is complete.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::output::OutputFormat;

/// `lab overseerr` arguments.
#[derive(Debug, Args)]
pub struct OverseerrArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
}

/// Run the `lab overseerr` subcommand stub.
///
/// # Errors
/// Always returns a not-yet-implemented message.
pub async fn run(_args: OverseerrArgs, _format: OutputFormat) -> Result<ExitCode> {
    anyhow::bail!("overseerr is not yet implemented — run `lab help` for available services")
}
