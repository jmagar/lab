//! Ratatui plugin manager. See `crates/lab/src/tui/CLAUDE.md` for the
//! TUI-vs-CLI divide and `.mcp.json` patching rules.

pub mod app;
pub mod metadata;

pub use app::run;
