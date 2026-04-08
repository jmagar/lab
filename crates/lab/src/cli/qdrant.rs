//! `lab qdrant` — CLI stub (not yet implemented).
//!
//! Replace this stub once `qdrant` SDK client is complete.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::output::OutputFormat;

/// `lab qdrant` arguments.
#[derive(Debug, Args)]
pub struct QdrantArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
}

/// Run the `lab qdrant` subcommand stub.
///
/// # Errors
/// Always returns a not-yet-implemented message.
pub async fn run(_args: QdrantArgs, _format: OutputFormat) -> Result<ExitCode> {
    anyhow::bail!("qdrant is not yet implemented — run `lab help` for available services")
}
