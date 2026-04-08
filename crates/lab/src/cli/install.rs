//! `lab install` / `lab uninstall` / `lab init`.
//!
//! These subcommands mutate the user's `.mcp.json` and/or `~/.lab/.env`.
//! Real logic lives in later plans — stubs just log intent.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

/// `lab install` arguments.
#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Services to install.
    #[arg(required = true)]
    pub services: Vec<String>,
}

/// `lab uninstall` arguments.
#[derive(Debug, Args)]
pub struct UninstallArgs {
    /// Services to uninstall.
    #[arg(required = true)]
    pub services: Vec<String>,
}

/// Run `lab install`. Stub.
pub fn run_install(args: InstallArgs) -> Result<ExitCode> {
    tracing::warn!(services = ?args.services, "lab install: not yet implemented");
    Ok(ExitCode::SUCCESS)
}

/// Run `lab uninstall`. Stub.
pub fn run_uninstall(args: UninstallArgs) -> Result<ExitCode> {
    tracing::warn!(services = ?args.services, "lab uninstall: not yet implemented");
    Ok(ExitCode::SUCCESS)
}

/// Run `lab init` setup wizard. Stub.
pub fn run_init() -> Result<ExitCode> {
    tracing::warn!("lab init: setup wizard not yet implemented");
    Ok(ExitCode::SUCCESS)
}
