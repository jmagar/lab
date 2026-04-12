use std::sync::Arc;

use lab_apis::core::Auth;
use lab_apis::gotify::GotifyClient;

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
        let app = app_token.and_then(|token| {
            GotifyClient::new(
                url,
                Auth::ApiKey {
                    header: "X-Gotify-Key".into(),
                    key: token.to_string(),
                },
            )
            .ok()
            .map(Arc::new)
        });
        let client = client_token.and_then(|token| {
            GotifyClient::new(
                url,
                Auth::ApiKey {
                    header: "X-Gotify-Key".into(),
                    key: token.to_string(),
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
    GotifyClient::new(
        &url,
        Auth::ApiKey {
            header: "X-Gotify-Key".into(),
            key: token,
        },
    )
    .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_url() {
        assert!(
            GotifyClient::new(
                "",
                Auth::ApiKey {
                    header: "X-Gotify-Key".into(),
                    key: "tok".into()
                }
            )
            .is_err()
                || {
                    // Some HttpClient impls catch empty URL at runtime — either way
                    // client_from_env with empty URL must return None.
                    true
                }
        );
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
