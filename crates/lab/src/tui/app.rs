//! TUI entry point. The full plugin manager UI is implemented in later
//! plans; this stub keeps the surface compilable and returns immediately.

use anyhow::Result;

/// Run the plugin manager TUI. Currently a no-op stub.
#[allow(clippy::unnecessary_wraps)]
pub fn run() -> Result<()> {
    tracing::warn!("tui::run() stub — plugin manager not yet implemented");
    Ok(())
}
