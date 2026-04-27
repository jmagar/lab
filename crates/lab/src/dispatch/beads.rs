//! Shared dispatch layer for the `beads` service.
//!
//! Read-only v1 — exposes `issue.list` and `issue.get` plus the standard
//! `help` / `schema` built-ins. Writes (create, update, close, comment) are
//! deferred until the read surface is validated end-to-end.
//!
//! MCP tool intentionally exposed: `issue.list` and `issue.get` are safe;
//! no destructive actions are present in v1.

mod catalog;
mod client;
mod dispatch;
mod params;

pub use catalog::ACTIONS;
pub use client::client_from_env;
pub use dispatch::{dispatch, dispatch_with_optional_client};
