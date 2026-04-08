//! `lab plugins` — open the plugin manager TUI.

use std::process::ExitCode;

use anyhow::Result;

/// Run the plugins subcommand.
pub fn run() -> Result<ExitCode> {
    crate::tui::run()?;
    Ok(ExitCode::SUCCESS)
}
