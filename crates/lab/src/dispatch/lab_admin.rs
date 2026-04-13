//! Shared dispatch layer for the internal `lab_admin` tool.
//!
//! This module is the single authoritative owner of the `lab_admin` action
//! catalog and shared dispatch semantics. No HTTP client is needed — the
//! underlying `audit_services` call is a local filesystem scan.

mod catalog;
mod dispatch;
mod params;

pub use catalog::ACTIONS;
pub use dispatch::dispatch;
