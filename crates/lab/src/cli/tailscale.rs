//! `lab tailscale` — CLI stub (not yet implemented).
//!
//! Replace this stub once `tailscale` SDK client is complete.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::output::OutputFormat;

/// `lab tailscale` arguments.
#[derive(Debug, Args)]
pub struct TailscaleArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
}

/// Run the `lab tailscale` subcommand stub.
///
/// # Errors
/// Always returns a not-yet-implemented message.
pub async fn run(_args: TailscaleArgs, _format: OutputFormat) -> Result<ExitCode> {
    anyhow::bail!("tailscale is not yet implemented — run `lab help` for available services")
}
