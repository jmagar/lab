//! `lab radarr` — CLI shim for the Radarr service.
//!
//! Thin shim: parse → call client → format. No business logic here.
//! See `crates/lab/src/cli/CLAUDE.md` for the shim rules.

use std::process::ExitCode;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::output::{OutputFormat, print};

/// `lab radarr` arguments.
#[derive(Debug, Args)]
pub struct RadarrArgs {
    #[command(subcommand)]
    pub command: RadarrCommand,
}

/// Radarr subcommands.
#[derive(Debug, Subcommand)]
pub enum RadarrCommand {
    /// Return Radarr system status and version.
    SystemStatus,
}

/// Run the `lab radarr` subcommand.
///
/// # Errors
/// Returns an error if the client is not configured or the API call fails.
pub async fn run(args: RadarrArgs, format: OutputFormat) -> Result<ExitCode> {
    let client = crate::mcp::services::radarr::client_from_env()
        .ok_or_else(|| anyhow::anyhow!("RADARR_URL and RADARR_API_KEY must be set"))?;

    match args.command {
        RadarrCommand::SystemStatus => {
            let status = client.system_status().await?;
            print(&status, format)?;
        }
    }

    Ok(ExitCode::SUCCESS)
}
