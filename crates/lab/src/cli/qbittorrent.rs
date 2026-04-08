//! `lab qbittorrent` — CLI stub (not yet implemented).
//!
//! Replace this stub once `qbittorrent` SDK client is complete.
//! See `radarr.rs` for the reference pattern.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::output::OutputFormat;

/// `lab qbittorrent` arguments.
#[derive(Debug, Args)]
pub struct QbittorrentArgs {
    /// Action to run (e.g. help).
    pub action: Option<String>,
}

/// Run the `lab qbittorrent` subcommand stub.
///
/// # Errors
/// Always returns a not-yet-implemented message.
pub async fn run(_args: QbittorrentArgs, _format: OutputFormat) -> Result<ExitCode> {
    anyhow::bail!("qbittorrent is not yet implemented — run `lab help` for available services")
}
