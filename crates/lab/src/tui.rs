//! Ratatui plugin manager. See `crates/lab/src/tui/CLAUDE.md` for the
//! TUI-vs-CLI divide and `.mcp.json` patching rules.

pub mod app;
pub mod display;
pub mod ecosystem;
pub mod events;
pub mod install;
pub mod marketplace;
pub mod mcp_patch;
pub mod metadata;
pub mod preview;
pub mod services;
pub mod state;
pub mod update;

pub use app::run;
