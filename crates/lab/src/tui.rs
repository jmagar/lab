//! Ratatui plugin manager. See `crates/lab/src/tui/CLAUDE.md` for the
//! TUI-vs-CLI divide and `.mcp.json` patching rules.

pub mod app;
pub mod display;
#[allow(dead_code)]
pub mod ecosystem;
#[allow(dead_code)]
pub mod events;
#[allow(dead_code)]
pub mod install;
#[allow(dead_code)]
pub mod marketplace;
pub mod mcp_patch;
pub mod metadata;
pub mod preview;
pub mod services;
#[allow(dead_code)]
pub mod state;
#[allow(dead_code)]
pub mod update;

pub use app::run;
