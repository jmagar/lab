//! `lab prowlarr` — CLI stub (not yet implemented).
//!
//! Replace this stub once `prowlarr` SDK client is complete.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::output::OutputFormat;

/// `lab prowlarr` arguments.
#[derive(Debug, Args)]
pub struct ProwlarrArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
}

/// Run the `lab prowlarr` subcommand stub.
///
/// # Errors
/// Always returns a not-yet-implemented message.
pub async fn run(_args: ProwlarrArgs, _format: OutputFormat) -> Result<ExitCode> {
    anyhow::bail!("prowlarr is not yet implemented — run `lab help` for available services")
}
