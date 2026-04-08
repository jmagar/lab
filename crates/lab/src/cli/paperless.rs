//! `lab paperless` — CLI stub (not yet implemented).
//!
//! Replace this stub once `paperless` SDK client is complete.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::output::OutputFormat;

/// `lab paperless` arguments.
#[derive(Debug, Args)]
pub struct PaperlessArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
}

/// Run the `lab paperless` subcommand stub.
///
/// # Errors
/// Always returns a not-yet-implemented message.
pub async fn run(_args: PaperlessArgs, _format: OutputFormat) -> Result<ExitCode> {
    anyhow::bail!("paperless is not yet implemented — run `lab help` for available services")
}
