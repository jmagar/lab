//! Output formatting for CLI commands.
//!
//! All CLI handlers should call [`print`] with their data — it picks
//! human-readable or JSON based on the active [`OutputFormat`].

#![allow(clippy::print_stdout)]

use anyhow::Result;
use serde::Serialize;

/// CLI output format, selected by the top-level `--json` flag.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable (tables, colors where supported).
    #[default]
    Human,
    /// Machine-readable JSON, one value per invocation.
    Json,
}

impl OutputFormat {
    /// Resolve the format from a boolean `--json` flag.
    #[must_use]
    pub const fn from_json_flag(json: bool) -> Self {
        if json { Self::Json } else { Self::Human }
    }
}

/// Print a serializable value in the requested format.
///
/// Human output falls back to pretty-printed JSON for now; individual
/// commands with richer rendering can use [`print_table`] directly.
pub fn print<T: Serialize>(value: &T, _format: OutputFormat) -> Result<()> {
    let rendered = serde_json::to_string_pretty(value)?;
    println!("{rendered}");
    Ok(())
}

/// Print a pre-built `tabled::Table` to stdout.
pub fn print_table(table: &tabled::Table) {
    println!("{table}");
}
