//! Beads error taxonomy.
//!
//! `BeadsError::redacted_message()` produces a generic user-facing string;
//! the full `sqlx::Error` is logged server-side at WARN/ERROR. Never surface
//! the `Display` form of the database error to MCP/HTTP clients — it can
//! contain table names, constraint details, and other server internals.

use thiserror::Error;

/// Errors produced by the beads service.
#[derive(Debug, Error)]
pub enum BeadsError {
    /// Pool/connection construction or query failure.
    #[error("database error")]
    Database(#[from] sqlx_core::Error),

    /// Issue id has no matching row.
    #[error("issue not found: {id}")]
    NotFound {
        /// The id that was looked up.
        id: String,
    },

    /// Port file could not be read.
    #[error("dolt port file unreadable: {path}")]
    PortFileError {
        /// Path attempted.
        path: String,
    },

    /// Port file content failed to parse as a u16.
    #[error("invalid port number in port file")]
    InvalidPort {
        /// Raw (untrusted) string read from the port file.
        raw: String,
    },

    /// Configured value invalid.
    #[error("invalid configuration: {0}")]
    Config(String),
}

impl BeadsError {
    /// Stable kind tag for envelope mapping.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::NotFound { .. } => "issue_not_found",
            Self::PortFileError { .. } | Self::InvalidPort { .. } => "beads_unavailable",
            Self::Database(_) | Self::Config(_) => "internal_error",
        }
    }

    /// Generic, user-safe message. Always preferred over `Display` at the
    /// MCP / HTTP envelope boundary. Full detail is logged server-side.
    #[must_use]
    pub fn redacted_message(&self) -> String {
        match self {
            Self::NotFound { id } => format!("issue not found: {id}"),
            Self::PortFileError { .. } | Self::InvalidPort { .. } => {
                "beads database is not reachable".to_string()
            }
            Self::Database(_) => "database error".to_string(),
            Self::Config(msg) => format!("invalid configuration: {msg}"),
        }
    }
}
