use lab_apis::qbittorrent::QbittorrentClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Perform a qBittorrent `WebUI` login and return the `SID=<value>` cookie string.
///
/// Uses reqwest directly because `HttpClient` does not expose `Set-Cookie` headers.
/// Parses the `SID` value out of the `Set-Cookie` response header manually since
/// the `cookies` reqwest feature is not enabled in this workspace.
async fn obtain_sid(base_url: &str, username: &str, password: &str) -> Option<String> {
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;
    let url = format!("{}/api/v2/auth/login", base_url.trim_end_matches('/'));
    let resp = client
        .post(&url)
        .form(&[("username", username), ("password", password)])
        .send()
        .await
        .ok()?;
    // Save the Set-Cookie header value before consuming the body.
    let set_cookie = resp
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let body = resp.text().await.ok()?;
    if body.trim() != "Ok." {
        tracing::warn!(url, "qbittorrent login rejected (check username/password)");
        return None;
    }
    // Header looks like: SID=abc123; Path=/; HttpOnly
    set_cookie.as_deref().and_then(|s| {
        s.split(';')
            .next()
            .filter(|part| part.trim_start().starts_with("SID="))
            .map(|part| part.trim().to_string())
    })
}

/// Build a `QbittorrentClient` from `QBITTORRENT_SID` only.
///
/// This is the sync path used by `AppState::ServiceClients::from_env()` at startup.
/// It only succeeds when `QBITTORRENT_SID` is pre-configured. If you only have
/// username/password, use `require_client()` from an async context instead.
///
/// Returns `None` if `QBITTORRENT_URL` or `QBITTORRENT_SID` is absent.
pub fn client_from_env() -> Option<QbittorrentClient> {
    let url = env_non_empty("QBITTORRENT_URL")?;
    let sid = env_non_empty("QBITTORRENT_SID")?;
    QbittorrentClient::new(&url, sid)
        .map_err(|e| tracing::warn!(error = %e, url, "qbittorrent client construction failed"))
        .ok()
}

/// Return a client or a structured error.
///
/// Tries `QBITTORRENT_SID` first; falls back to logging in with
/// `QBITTORRENT_USERNAME` + `QBITTORRENT_PASSWORD`.
///
/// # Errors
/// Returns `ToolError` if URL is missing, credentials are absent/wrong, or
/// client construction fails.
pub async fn require_client() -> Result<QbittorrentClient, ToolError> {
    let url = env_non_empty("QBITTORRENT_URL").ok_or_else(not_configured_error)?;
    let sid = if let Some(s) = env_non_empty("QBITTORRENT_SID") {
        s
    } else {
        let username = env_non_empty("QBITTORRENT_USERNAME").ok_or_else(not_configured_error)?;
        let password = env_non_empty("QBITTORRENT_PASSWORD").ok_or_else(not_configured_error)?;
        obtain_sid(&url, &username, &password)
            .await
            .ok_or_else(|| ToolError::Sdk {
                sdk_kind: "auth_failed".to_string(),
                message: "qbittorrent login failed — check QBITTORRENT_USERNAME/PASSWORD".to_string(),
            })?
    };
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
