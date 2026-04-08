//! `lab openai` — CLI stub (not yet implemented).
//!
//! Replace this stub once `openai` SDK client is complete.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::output::OutputFormat;

/// `lab openai` arguments.
#[derive(Debug, Args)]
pub struct OpenaiArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
}

/// Run the `lab openai` subcommand stub.
///
/// # Errors
/// Always returns a not-yet-implemented message.
pub async fn run(_args: OpenaiArgs, _format: OutputFormat) -> Result<ExitCode> {
    anyhow::bail!("openai is not yet implemented — run `lab help` for available services")
}
