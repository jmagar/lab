//! `lab sabnzbd` — CLI stub (not yet implemented).
//!
//! Replace this stub once `sabnzbd` SDK client is complete.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::output::OutputFormat;

/// `lab sabnzbd` arguments.
#[derive(Debug, Args)]
pub struct SabnzbdArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
}

/// Run the `lab sabnzbd` subcommand stub.
///
/// # Errors
/// Always returns a not-yet-implemented message.
pub async fn run(_args: SabnzbdArgs, _format: OutputFormat) -> Result<ExitCode> {
    anyhow::bail!("sabnzbd is not yet implemented — run `lab help` for available services")
}
