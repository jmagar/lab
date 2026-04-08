//! `lab tei` — CLI stub (not yet implemented).
//!
//! Replace this stub once `tei` SDK client is complete.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::output::OutputFormat;

/// `lab tei` arguments.
#[derive(Debug, Args)]
pub struct TeiArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
}

/// Run the `lab tei` subcommand stub.
///
/// # Errors
/// Always returns a not-yet-implemented message.
pub async fn run(_args: TeiArgs, _format: OutputFormat) -> Result<ExitCode> {
    anyhow::bail!("tei is not yet implemented — run `lab help` for available services")
}
