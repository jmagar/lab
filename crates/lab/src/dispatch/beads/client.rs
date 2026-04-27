//! Beads client construction.
//!
//! The client is built once at startup by `ServiceClients::from_env()` and
//! shared via `Arc<BeadsClient>` for the lifetime of the process.
//!
//! # Port resolution
//!
//! Order of precedence:
//! 1. `BEADS_DOLT_PORT` env var
//! 2. `BEADS_PORT_FILE` env var (path to a file containing the port)
//! 3. `.beads/dolt-server.port` relative to CWD
//!
//! The host defaults to `127.0.0.1` and may be overridden via
//! `BEADS_DOLT_HOST`. The DSN is constructed against the `lab` database with
//! the `root` user and no password — matching the bd tool's local Dolt
//! defaults.
//!
//! Port-file parsing happens at startup (synchronous), but pool construction
//! is async. We block on a small tokio runtime so `from_env` can stay sync
//! and match the convention used by every other service.

use std::path::PathBuf;

use lab_apis::beads::BeadsClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_DB: &str = "lab";
const DEFAULT_USER: &str = "root";
const DEFAULT_PORT_FILE: &str = ".beads/dolt-server.port";

/// Resolve the Dolt MySQL DSN from environment / port file.
///
/// Returns `None` when the port cannot be discovered — the service is
/// considered unavailable and downstream calls return `beads_unavailable`.
fn resolve_dsn() -> Option<String> {
    let host = env_non_empty("BEADS_DOLT_HOST").unwrap_or_else(|| DEFAULT_HOST.to_string());

    let port: u16 = if let Some(raw) = env_non_empty("BEADS_DOLT_PORT") {
        match raw.trim().parse::<u16>() {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(error = %e, "BEADS_DOLT_PORT is not a valid u16");
                return None;
            }
        }
    } else {
        let path = env_non_empty("BEADS_PORT_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_PORT_FILE));
        match std::fs::read_to_string(&path) {
            Ok(raw) => match raw.trim().parse::<u16>() {
                Ok(p) => p,
                Err(_) => {
                    tracing::warn!(
                        path = %path.display(),
                        "beads port file content is not a valid u16"
                    );
                    return None;
                }
            },
            Err(e) => {
                tracing::debug!(
                    path = %path.display(),
                    error = %e,
                    "beads port file unreadable; service will be marked unavailable"
                );
                return None;
            }
        }
    };

    Some(format!("mysql://{DEFAULT_USER}@{host}:{port}/{DEFAULT_DB}"))
}

/// Build the beads client once at startup.
///
/// Returns `None` when the Dolt port cannot be discovered. Retries are
/// not attempted — startup is the only resolution point, matching the
/// convention used by every other service in `lab`.
#[must_use]
pub fn client_from_env() -> Option<BeadsClient> {
    let dsn = resolve_dsn()?;
    // The pool is constructed lazily on first query in sqlx 0.8 — calling
    // `MySqlPoolOptions::connect_lazy` returns immediately and avoids
    // blocking startup on a Dolt round-trip. `BeadsClient::connect` performs
    // a real connect; we use `connect_lazy_with` here via a small helper.
    match BeadsClient::lazy(&dsn) {
        Ok(client) => Some(client),
        Err(e) => {
            tracing::warn!(error = %e, "beads client construction failed");
            None
        }
    }
}

/// Structured error returned when beads is not configured / not reachable.
pub fn unavailable_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "beads_unavailable".to_string(),
        message: "beads database is not reachable (dolt-server.port not found)".to_string(),
    }
}
