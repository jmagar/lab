//! `lab plugins` — open the plugin manager TUI.

use std::process::ExitCode;

/// Run the plugins subcommand.
pub fn run() -> ExitCode {
    crate::tui::run();
    ExitCode::SUCCESS
}
