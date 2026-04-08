//! `lab linkding` — CLI stub (not yet implemented).
//!
//! Replace this stub once `linkding` SDK client is complete.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::output::OutputFormat;

/// `lab linkding` arguments.
#[derive(Debug, Args)]
pub struct LinkdingArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
}

/// Run the `lab linkding` subcommand stub.
///
/// # Errors
/// Always returns a not-yet-implemented message.
pub async fn run(_args: LinkdingArgs, _format: OutputFormat) -> Result<ExitCode> {
    anyhow::bail!("linkding is not yet implemented — run `lab help` for available services")
}
