//! `lab install` / `lab uninstall` / `lab init`.
//!
//! These subcommands mutate the user's `.mcp.json` and/or `~/.lab/.env`.
//! Real logic lives in later plans — stubs return a clear not-implemented error.

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
///
/// # Errors
/// Always returns a not-yet-implemented error.
pub fn run_install(_args: &InstallArgs) -> Result<()> {
    anyhow::bail!("lab install: not yet implemented")
}

/// Run `lab uninstall`. Stub.
///
/// # Errors
/// Always returns a not-yet-implemented error.
pub fn run_uninstall(_args: &UninstallArgs) -> Result<()> {
    anyhow::bail!("lab uninstall: not yet implemented")
}

/// Run `lab init` setup wizard. Stub.
///
/// # Errors
/// Always returns a not-yet-implemented error.
pub fn run_init() -> Result<()> {
    anyhow::bail!("lab init: setup wizard not yet implemented")
}
