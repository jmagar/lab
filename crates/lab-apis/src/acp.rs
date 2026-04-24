//! Agent Client Protocol (ACP) — shared types, error taxonomy, persistence
//! trait, and provider types for the ACP runtime.
//!
//! This module is always-on (no feature flag). It owns the stable public
//! surface that both `lab-apis` consumers and the `lab` binary depend on:
//!
//! - [`types`] — session, agent, and message types (populated by lab-jwbg.2)
//! - [`error`] — `AcpError` (thiserror)
//! - [`persistence`] — `AcpPersistence` trait; implementation (SQLite) lives in `lab`
//! - [`session`] — `SessionHandle` and related provider types
//!
//! Stateful runtime (registry, SQLite persistence implementation, subprocess
//! launch) lives in `crates/lab/src/acp/`, not here.

/// ACP request/response and domain types.
pub mod types;

/// `AcpError` — typed errors (thiserror).
pub mod error;

/// `AcpPersistence` — storage trait (implementation lives in `lab`).
pub mod persistence;

/// `SessionHandle` and related provider types.
pub mod session;

// Convenience re-exports of the canonical public surface.
pub use error::{AcpError, PersistenceError};
pub use session::{SessionCommand, SessionError, SessionHandle};
pub use types::{
    AcpContentBlock, AcpEvent, AcpPermissionOption, AcpProviderHealth, AcpSessionState,
    AcpSessionSummary,
};
