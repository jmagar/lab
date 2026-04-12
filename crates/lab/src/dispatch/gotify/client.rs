use std::sync::Arc;

use lab_apis::core::Auth;
use lab_apis::gotify::GotifyClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

#[derive(Clone)]
pub struct GotifyClients {
    health: Arc<GotifyClient>,
    app: Option<Arc<GotifyClient>>,
    client: Option<Arc<GotifyClient>>,
}

impl GotifyClients {
    fn new(url: &str, app_token: Option<&str>, client_token: Option<&str>) -> Option<Self> {
        let health = Arc::new(GotifyClient::new(url, Auth::None).ok()?);
        let app = app_token.and_then(|t| {
            GotifyClient::new(
                url,
                Auth::ApiKey {
                    header: "X-Gotify-Key".into(),
                    key: t.to_string(),
                },
            )
            .ok()
            .map(Arc::new)
        });
        let client = client_token.and_then(|t| {
            GotifyClient::new(
                url,
                Auth::ApiKey {
                    header: "X-Gotify-Key".into(),
                    key: t.to_string(),
                },
            )
            .ok()
            .map(Arc::new)
        });
        Some(Self {
            health,
            app,
            client,
        })
    }

    #[must_use]
    pub fn health(&self) -> &GotifyClient {
        self.health.as_ref()
    }

    #[must_use]
    pub fn app(&self) -> Option<&GotifyClient> {
        self.app.as_deref()
    }

    #[must_use]
    pub fn client(&self) -> Option<&GotifyClient> {
        self.client.as_deref()
    }
}

fn resolve_tokens() -> (Option<String>, Option<String>) {
    let legacy = env_non_empty("GOTIFY_TOKEN");
    let app = env_non_empty("GOTIFY_APP_TOKEN").or_else(|| legacy.clone());
    let client = env_non_empty("GOTIFY_CLIENT_TOKEN").or(legacy);
    (app, client)
}

/// Build Gotify clients from the default-instance env vars.
///
/// Returns `None` when `GOTIFY_URL` is absent or invalid.
pub fn clients_from_env() -> Option<GotifyClients> {
    let url = env_non_empty("GOTIFY_URL")?;
    let (app_token, client_token) = resolve_tokens();
    GotifyClients::new(&url, app_token.as_deref(), client_token.as_deref())
}

/// Build the management-scoped Gotify client from env.
///
/// Prefers `GOTIFY_CLIENT_TOKEN`, then falls back to legacy `GOTIFY_TOKEN`.
pub fn client_from_env() -> Option<GotifyClient> {
    let url = env_non_empty("GOTIFY_URL")?;
    let (_, client_token) = resolve_tokens();
    let token = client_token?;
    client_from_vars(Some(&url), Some(&token))
}

/// Return a management-scoped client or a structured `internal_error` if not configured.
///
/// Intended for callers that need only the management client (e.g. `lab doctor`).
/// The dispatch layer uses `clients_from_env()` + `not_configured_error()` because it
/// requires all three client roles (`health`, `app`, `client`).
#[allow(dead_code)]
pub fn require_client() -> Result<GotifyClient, ToolError> {
    client_from_env().ok_or_else(not_configured_error)
}

/// Structured error returned when Gotify env vars are absent.
///
/// Exposed so that callers holding a pre-built `Option<GotifyClients>` (e.g. the API
/// handler) can produce the same error without calling `require_client()` redundantly.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "GOTIFY_URL or GOTIFY_TOKEN not configured".into(),
    }
}

pub fn client_from_vars(url: Option<&str>, token: Option<&str>) -> Option<GotifyClient> {
    let url = url.filter(|v| !v.is_empty())?;
    let token = token.filter(|v| !v.is_empty())?;
    GotifyClient::new(
        url,
        Auth::ApiKey {
            header: "X-Gotify-Key".into(),
            key: token.to_string(),
        },
    )
    .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clients_from_env_returns_none_without_url() {
        assert!(client_from_vars(None, Some("token")).is_none());
        assert!(client_from_vars(Some(""), Some("token")).is_none());
        assert!(client_from_vars(Some("http://localhost"), None).is_none());
        assert!(client_from_vars(Some("http://localhost"), Some("")).is_none());
    }

    #[test]
    fn token_resolution_uses_legacy_token_as_fallback() {
        let vars = [
            ("GOTIFY_TOKEN", "legacy"),
            ("GOTIFY_APP_TOKEN", ""),
            ("GOTIFY_CLIENT_TOKEN", ""),
        ];
        let get = |name: &str| {
            vars.iter()
                .find_map(|(key, value)| (*key == name).then(|| (*value).to_string()))
                .filter(|value| !value.is_empty())
        };
        let legacy = get("GOTIFY_TOKEN");
        let app = get("GOTIFY_APP_TOKEN").or_else(|| legacy.clone());
        let client = get("GOTIFY_CLIENT_TOKEN").or(legacy);
        assert_eq!(app.as_deref(), Some("legacy"));
        assert_eq!(client.as_deref(), Some("legacy"));
    }

    #[test]
    fn token_resolution_prefers_scoped_tokens() {
        let vars = [
            ("GOTIFY_TOKEN", "legacy"),
            ("GOTIFY_APP_TOKEN", "app"),
            ("GOTIFY_CLIENT_TOKEN", "client"),
        ];
        let get = |name: &str| {
            vars.iter()
                .find_map(|(key, value)| (*key == name).then(|| (*value).to_string()))
                .filter(|value| !value.is_empty())
        };
        let legacy = get("GOTIFY_TOKEN");
        let app = get("GOTIFY_APP_TOKEN").or_else(|| legacy.clone());
        let client = get("GOTIFY_CLIENT_TOKEN").or(legacy);
        assert_eq!(app.as_deref(), Some("app"));
        assert_eq!(client.as_deref(), Some("client"));
    }
}
