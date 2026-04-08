//! `lab unifi` — CLI stub (not yet implemented).
//!
//! Replace this stub once `unifi` SDK client is complete.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::output::OutputFormat;

/// `lab unifi` arguments.
#[derive(Debug, Args)]
pub struct UnifiArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
}

/// Run the `lab unifi` subcommand stub.
///
/// # Errors
/// Always returns a not-yet-implemented message.
pub async fn run(_args: UnifiArgs, _format: OutputFormat) -> Result<ExitCode> {
    anyhow::bail!("unifi is not yet implemented — run `lab help` for available services")
}
