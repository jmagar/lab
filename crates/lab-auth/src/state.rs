use std::sync::Arc;

use tracing::info;
use url::Url;

use crate::config::{AuthConfig, AuthMode};
use crate::error::AuthError;
use crate::google::GoogleProvider;
use crate::jwt::SigningKeys;
use crate::sqlite::SqliteStore;

#[derive(Clone)]
pub struct AuthState {
    pub config: Arc<AuthConfig>,
    pub store: SqliteStore,
    pub signing_keys: Arc<SigningKeys>,
    pub google: Arc<GoogleProvider>,
}

impl AuthState {
    pub async fn new(config: AuthConfig) -> Result<Self, AuthError> {
        if !matches!(config.mode, AuthMode::OAuth) {
            return Err(AuthError::Config(
                "AuthState requires LAB_AUTH_MODE=oauth".to_string(),
            ));
        }

        let public_url = config.public_url.clone().ok_or_else(|| {
            AuthError::Config("LAB_PUBLIC_URL is required when LAB_AUTH_MODE=oauth".to_string())
        })?;
        let redirect_uri = build_google_redirect_uri(&public_url, &config.google.callback_path);
        let store = SqliteStore::open(config.sqlite_path.clone()).await?;
        let signing_keys = SigningKeys::load_or_create(&config.key_path)?;
        let mut google = GoogleProvider::new(
            config.google.client_id.clone(),
            config.google.client_secret.clone(),
            redirect_uri,
        )?;
        google.scopes.clone_from(&config.google.scopes);
        info!(
            auth_mode = "oauth",
            public_url = %public_url,
            google_redirect_uri = %google.redirect_uri,
            sqlite_path = %config.sqlite_path.display(),
            key_path = %config.key_path.display(),
            google_scopes = ?config.google.scopes,
            "lab-auth state initialized"
        );

        Ok(Self {
            config: Arc::new(config),
            store,
            signing_keys: Arc::new(signing_keys),
            google: Arc::new(google),
        })
    }

    /// Rate-limit guard for `/authorize` and `/browser_login` endpoints.
    /// Currently a no-op placeholder — returns Ok(()) unconditionally.
    pub fn check_authorize_rate_limit(&self) -> Result<(), crate::error::AuthError> {
        Ok(())
    }

    /// Rate-limit guard for `/register` endpoint.
    /// Currently a no-op placeholder — returns Ok(()) unconditionally.
    pub fn check_register_rate_limit(&self) -> Result<(), crate::error::AuthError> {
        Ok(())
    }

    /// Rejects new OAuth state rows when the pending count exceeds `max_pending_oauth_states`.
    pub async fn ensure_pending_oauth_state_capacity(
        &self,
    ) -> Result<(), crate::error::AuthError> {
        let count = self.store.count_pending_oauth_states().await?;
        if count >= self.config.max_pending_oauth_states {
            return Err(crate::error::AuthError::RateLimited {
                message: "too many pending authorization requests".to_string(),
                retry_after_ms: 5_000,
            });
        }
        Ok(())
    }

    #[cfg(test)]
    pub fn for_tests(
        config: AuthConfig,
        store: SqliteStore,
        signing_keys: SigningKeys,
        google: GoogleProvider,
    ) -> Self {
        Self {
            config: Arc::new(config),
            store,
            signing_keys: Arc::new(signing_keys),
            google: Arc::new(google),
        }
    }
}

fn build_google_redirect_uri(public_url: &Url, callback_path: &str) -> Url {
    let mut redirect_uri = public_url.clone();
    let base_path = redirect_uri.path().trim_end_matches('/');
    let callback_path = callback_path.trim_start_matches('/');
    let next_path = if base_path.is_empty() {
        format!("/{callback_path}")
    } else {
        format!("{base_path}/{callback_path}")
    };

    redirect_uri.set_path(&next_path);
    redirect_uri.set_query(None);
    redirect_uri.set_fragment(None);
    redirect_uri
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tempfile::tempdir;

    use super::*;
    use crate::config::GoogleConfig;

    #[tokio::test]
    async fn auth_state_preserves_public_url_path_prefix_in_google_redirect_uri() {
        let temp = tempdir().expect("tempdir");
        let state = AuthState::new(AuthConfig {
            mode: AuthMode::OAuth,
            public_url: Some(Url::parse("https://lab.example.com/gateway").expect("public url")),
            sqlite_path: temp.path().join("auth.db"),
            key_path: temp.path().join("auth.pem"),
            bootstrap_secret: None,
            allowed_client_redirect_uris: Vec::new(),
            google: GoogleConfig {
                client_id: "client-id".to_string(),
                client_secret: "client-secret".to_string(),
                callback_path: "/auth/google/callback".to_string(),
                scopes: vec![
                    "openid".to_string(),
                    "email".to_string(),
                    "profile".to_string(),
                ],
            },
            access_token_ttl: Duration::from_secs(3600),
            refresh_token_ttl: Duration::from_secs(3600),
            auth_code_ttl: Duration::from_secs(300),
        })
        .await
        .expect("auth state");

        assert_eq!(
            state.google.redirect_uri.as_str(),
            "https://lab.example.com/gateway/auth/google/callback"
        );
    }
}
