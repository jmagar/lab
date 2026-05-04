//! `labby install` / `labby uninstall` / `labby init`.
//!
//! These subcommands mutate the user's `.mcp.json` and/or `~/.lab/.env`.
//! Real logic lives in later plans — stubs return a clear not-implemented error.

use anyhow::Result;
use clap::Args;

/// `labby install` arguments.
#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Services to install.
    #[arg(required = true)]
    pub services: Vec<String>,
}

/// `labby uninstall` arguments.
#[derive(Debug, Args)]
pub struct UninstallArgs {
    /// Services to uninstall.
    #[arg(required = true)]
    pub services: Vec<String>,
}

/// Run `labby install`. Stub.
///
/// # Errors
/// Always returns a not-yet-implemented error.
pub fn run_install(_args: &InstallArgs) -> Result<()> {
    anyhow::bail!("labby install: not yet implemented")
}

/// Run `labby uninstall`. Stub.
///
/// # Errors
/// Always returns a not-yet-implemented error.
pub fn run_uninstall(_args: &UninstallArgs) -> Result<()> {
    anyhow::bail!("labby uninstall: not yet implemented")
}

/// Run `labby init` setup wizard. Stub.
///
/// # Errors
/// Always returns a not-yet-implemented error.
pub fn run_init() -> Result<()> {
    anyhow::bail!("labby init: setup wizard not yet implemented")
}
