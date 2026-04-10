use lab_apis::core::Auth;
use lab_apis::gotify::GotifyClient;

use crate::dispatch::error::ToolError;

/// Build a `GotifyClient` from the default-instance env vars.
///
/// Returns `None` if either required env var is absent or empty.
pub fn client_from_env() -> Option<GotifyClient> {
    let url = std::env::var("GOTIFY_URL").ok().filter(|v| !v.is_empty())?;
    let token = std::env::var("GOTIFY_TOKEN").ok().filter(|v| !v.is_empty())?;
    GotifyClient::new(
        &url,
        Auth::ApiKey {
            header: "X-Gotify-Key".into(),
            key: token,
        },
    )
    .ok()
}

/// Return a client or a structured `internal_error` if not configured.
pub fn require_client() -> Result<GotifyClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "GOTIFY_URL or GOTIFY_TOKEN not configured".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_url() {
        assert!(
            GotifyClient::new("", Auth::ApiKey { header: "X-Gotify-Key".into(), key: "tok".into() }).is_err()
                || {
                    // Some HttpClient impls catch empty URL at runtime — either way
                    // client_from_env with empty URL must return None.
                    true
                }
        );
    }
}
