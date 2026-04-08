//! `lab plugins` — open the plugin manager TUI.

use std::process::ExitCode;

/// Run the plugins subcommand.
pub fn run() -> anyhow::Result<ExitCode> {
    crate::tui::run()?;
    Ok(ExitCode::SUCCESS)
}
