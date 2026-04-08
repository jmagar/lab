//! `lab plex` — CLI stub (not yet implemented).
//!
//! Replace this stub once `plex` SDK client is complete.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::output::OutputFormat;

/// `lab plex` arguments.
#[derive(Debug, Args)]
pub struct PlexArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
}

/// Run the `lab plex` subcommand stub.
///
/// # Errors
/// Always returns a not-yet-implemented message.
pub async fn run(_args: PlexArgs, _format: OutputFormat) -> Result<ExitCode> {
    anyhow::bail!("plex is not yet implemented — run `lab help` for available services")
}
