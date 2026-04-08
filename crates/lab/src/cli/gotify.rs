//! `lab gotify` — CLI stub (not yet implemented).
//!
//! Replace this stub once `gotify` SDK client is complete.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::output::OutputFormat;

/// `lab gotify` arguments.
#[derive(Debug, Args)]
pub struct GotifyArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
}

/// Run the `lab gotify` subcommand stub.
///
/// # Errors
/// Always returns a not-yet-implemented message.
pub async fn run(_args: GotifyArgs, _format: OutputFormat) -> Result<ExitCode> {
    anyhow::bail!("gotify is not yet implemented — run `lab help` for available services")
}
