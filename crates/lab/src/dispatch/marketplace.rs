//! Shared dispatch layer for the `marketplace` synthetic service.
//!
//! Reads marketplace registrations from `~/.claude/plugins/known_marketplaces.json`
//! and installed-plugin state from `~/.claude/plugins/installed_plugins.json`,
//! merges with each marketplace's `.claude-plugin/marketplace.json`, and shells
//! out to `claude plugin install/uninstall` for destructive operations.

mod acp_catalog;
mod acp_client;
mod acp_dispatch;
mod catalog;
mod client;
mod dispatch;
mod params;

pub use catalog::ACTIONS;
pub use dispatch::dispatch;
