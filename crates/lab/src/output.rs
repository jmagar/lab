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
/// `Json` emits compact single-line JSON (machine-readable).
/// `Human` emits pretty-printed JSON as a fallback; individual commands
/// with richer rendering should use [`print_table`] directly.
pub fn print<T: Serialize>(value: &T, format: OutputFormat) -> Result<()> {
    println!("{}", render(value, format)?);
    Ok(())
}

/// Render a serializable value to a string in the requested format.
///
/// Used by [`print`] and available for testing without stdout capture.
pub fn render<T: Serialize>(value: &T, format: OutputFormat) -> Result<String> {
    Ok(match format {
        OutputFormat::Human => serde_json::to_string_pretty(value)?,
        OutputFormat::Json => serde_json::to_string(value)?,
    })
}

/// Print a pre-built `tabled::Table` to stdout.
pub fn print_table(table: &tabled::Table) {
    println!("{table}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn json_format_is_compact() {
        let val = json!({"name": "test", "count": 42});
        let out = render(&val, OutputFormat::Json).unwrap();
        assert!(!out.contains('\n'), "Json format must be single-line");
    }

    #[test]
    fn human_format_is_pretty() {
        let val = json!({"name": "test", "count": 42});
        let out = render(&val, OutputFormat::Human).unwrap();
        assert!(out.contains('\n'), "Human format must be multi-line");
    }

    #[test]
    fn formats_differ() {
        let val = json!({"a": 1});
        let human = render(&val, OutputFormat::Human).unwrap();
        let json = render(&val, OutputFormat::Json).unwrap();
        assert_ne!(human, json, "Human and Json must produce different output");
    }

    #[test]
    fn from_json_flag() {
        assert_eq!(OutputFormat::from_json_flag(true), OutputFormat::Json);
        assert_eq!(OutputFormat::from_json_flag(false), OutputFormat::Human);
    }
}
