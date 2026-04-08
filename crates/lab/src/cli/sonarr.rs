//! `lab sonarr` — CLI stub (not yet implemented).
//!
//! Replace this stub once `sonarr` SDK client is complete.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::output::OutputFormat;

/// `lab sonarr` arguments.
#[derive(Debug, Args)]
pub struct SonarrArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
}

/// Run the `lab sonarr` subcommand stub.
///
/// # Errors
/// Always returns a not-yet-implemented message.
pub async fn run(_args: SonarrArgs, _format: OutputFormat) -> Result<ExitCode> {
    anyhow::bail!("sonarr is not yet implemented — run `lab help` for available services")
}
