//! Upstream OAuth client integration for proxied MCP servers.
//!
//! This module is a STUB placeholder introduced by Task 0 (rmcp integration PoC).
//! It is intentionally NOT wired into `oauth.rs` yet — Task 2 will declare
//! `pub mod upstream;` in `oauth.rs` once the real implementation lands.
//!
//! Submodules:
//! - [`refresh`] — documents findings from the Task 0 gating spike and will
//!   hold the refresh-on-401 + reauthorization helpers for Task 2.

pub mod refresh;
