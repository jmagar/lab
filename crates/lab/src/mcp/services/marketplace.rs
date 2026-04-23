//! MCP adapter for the `marketplace` tool.
//!
//! Thin re-export; all logic lives in `crate::dispatch::marketplace`.

pub use crate::dispatch::marketplace::ACTIONS;
pub use crate::dispatch::marketplace::dispatch;
