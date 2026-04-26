//! Shared dispatch layer for the `stash` service.
//!
//! Stash manages versioned, provider-synced component snapshots stored under
//! the configured workspace root (`[workspace].root` in `config.toml`, falling
//! back to `~/.lab/stash`).
//!
//! Surfaces (MCP, CLI, API) are thin adapters over the modules here.

pub mod catalog;
pub mod client;
pub mod dispatch;
pub mod import;
pub mod params;
pub mod store;

#[allow(unused_imports)]
pub use dispatch::dispatch;
