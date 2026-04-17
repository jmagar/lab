use std::sync::Arc;

use tracing::info;

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
        let redirect_uri = public_url
            .join(config.google.callback_path.trim_start_matches('/'))
            .map_err(|error| {
                AuthError::Config(format!(
                    "build google redirect URI from LAB_PUBLIC_URL: {error}"
                ))
            })?;
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
