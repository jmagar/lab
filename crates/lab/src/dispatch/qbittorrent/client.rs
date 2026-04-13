use lab_apis::qbittorrent::QbittorrentClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build a `QbittorrentClient` from default-instance env vars.
///
/// qBittorrent uses cookie-based auth. The SID cookie can be pre-supplied via
/// `QBITTORRENT_SID` env var (format: `SID=<value>`). If not set, we construct
/// a bare client with a placeholder cookie — the caller should perform login
/// separately. In practice, operators should set `QBITTORRENT_SID` after
/// authenticating once.
///
/// Returns `None` if `QBITTORRENT_URL` is absent or empty.
/// Called by `AppState` at startup — keep pure (no side effects, no logging).
pub fn client_from_env() -> Option<QbittorrentClient> {
    let url = env_non_empty("QBITTORRENT_URL")?;
    // Use pre-obtained SID cookie if available, otherwise attempt with empty cookie.
    // Operators should configure QBITTORRENT_SID for proper auth.
    let sid = env_non_empty("QBITTORRENT_SID").unwrap_or_default();
    QbittorrentClient::new(&url, sid)
        .map_err(|e| tracing::warn!(error = %e, url, "qbittorrent client construction failed"))
        .ok()
}

/// Return a client or a structured error.
///
/// # Errors
/// Returns a `ToolError` if `QBITTORRENT_URL` is not configured or
/// if client construction fails.
pub fn require_client() -> Result<QbittorrentClient, ToolError> {
    let url = env_non_empty("QBITTORRENT_URL").ok_or_else(not_configured_error)?;
    let sid = env_non_empty("QBITTORRENT_SID").unwrap_or_default();
    QbittorrentClient::new(&url, sid).map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: format!("qbittorrent client init failed: {e}"),
    })
}

/// Structured error for callers that hold a pre-built `Option<QbittorrentClient>`.
/// The API handler calls this directly instead of re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "QBITTORRENT_URL not configured".to_string(),
    }
}
