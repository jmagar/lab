//! `lab tautulli` — CLI stub (not yet implemented).
//!
//! Replace this stub once `tautulli` SDK client is complete.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::output::OutputFormat;

/// `lab tautulli` arguments.
#[derive(Debug, Args)]
pub struct TautulliArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
}

/// Run the `lab tautulli` subcommand stub.
///
/// # Errors
/// Always returns a not-yet-implemented message.
pub async fn run(_args: TautulliArgs, _format: OutputFormat) -> Result<ExitCode> {
    anyhow::bail!("tautulli is not yet implemented — run `lab help` for available services")
}
