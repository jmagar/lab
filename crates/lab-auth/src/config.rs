use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::AuthError;

const DEFAULT_CALLBACK_PATH: &str = "/auth/google/callback";
const DEFAULT_AUTH_DB_NAME: &str = "auth.db";
const DEFAULT_KEY_NAME: &str = "auth-jwt.pem";
const DEFAULT_ACCESS_TOKEN_TTL_SECS: u64 = 3600;
const DEFAULT_REFRESH_TOKEN_TTL_SECS: u64 = 30 * 24 * 3600;
const DEFAULT_AUTH_CODE_TTL_SECS: u64 = 300;
const DEFAULT_REGISTER_REQUESTS_PER_MINUTE: u32 = 20;
const DEFAULT_AUTHORIZE_REQUESTS_PER_MINUTE: u32 = 60;
const DEFAULT_MAX_PENDING_OAUTH_STATES: usize = 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    #[default]
    Bearer,
    OAuth,
}

impl AuthMode {
    fn parse(value: Option<&str>) -> Result<Self, AuthError> {
        match value
            .unwrap_or("bearer")
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "bearer" => Ok(Self::Bearer),
            "oauth" => Ok(Self::OAuth),
            other => Err(AuthError::Config(format!(
                "LAB_AUTH_MODE must be `bearer` or `oauth`, got `{other}`"
            ))),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AuthModeConfig {
    pub mode: AuthMode,
}

impl AuthModeConfig {
    pub fn from_sources(
        vars: impl IntoIterator<Item = (String, String)>,
    ) -> Result<Self, AuthError> {
        let vars = normalize(vars);
        Ok(Self {
            mode: AuthMode::parse(vars.get("LAB_AUTH_MODE").map(String::as_str))?,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoogleConfig {
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,
    #[serde(default = "default_callback_path")]
    pub callback_path: String,
    #[serde(default = "default_google_scopes")]
    pub scopes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthConfig {
    pub mode: AuthMode,
    pub public_url: Option<Url>,
    pub sqlite_path: PathBuf,
    pub key_path: PathBuf,
    pub bootstrap_secret: Option<String>,
    pub allowed_client_redirect_uris: Vec<String>,
    pub allowed_emails: Vec<String>,
    pub google: GoogleConfig,
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,
    pub auth_code_ttl: Duration,
    pub register_requests_per_minute: u32,
    pub authorize_requests_per_minute: u32,
    pub max_pending_oauth_states: usize,
}

impl Default for AuthConfig {
    fn default() -> Self {
        let base_dir = default_auth_dir();
        Self {
            mode: AuthMode::Bearer,
            public_url: None,
            sqlite_path: base_dir.join(DEFAULT_AUTH_DB_NAME),
            key_path: base_dir.join(DEFAULT_KEY_NAME),
            bootstrap_secret: None,
            allowed_client_redirect_uris: Vec::new(),
            allowed_emails: Vec::new(),
            google: GoogleConfig::default(),
            access_token_ttl: Duration::from_secs(DEFAULT_ACCESS_TOKEN_TTL_SECS),
            refresh_token_ttl: Duration::from_secs(DEFAULT_REFRESH_TOKEN_TTL_SECS),
            auth_code_ttl: Duration::from_secs(DEFAULT_AUTH_CODE_TTL_SECS),
            register_requests_per_minute: DEFAULT_REGISTER_REQUESTS_PER_MINUTE,
            authorize_requests_per_minute: DEFAULT_AUTHORIZE_REQUESTS_PER_MINUTE,
            max_pending_oauth_states: DEFAULT_MAX_PENDING_OAUTH_STATES,
        }
    }
}

impl AuthConfig {
    pub fn from_sources(
        vars: impl IntoIterator<Item = (String, String)>,
    ) -> Result<Self, AuthError> {
        let vars = normalize(vars);
        let mode = AuthMode::parse(vars.get("LAB_AUTH_MODE").map(String::as_str))?;
        let config = Self {
            mode,
            public_url: read_url(&vars, "LAB_PUBLIC_URL")?,
            sqlite_path: read_path(&vars, "LAB_AUTH_SQLITE_PATH")
                .unwrap_or_else(|| default_auth_dir().join(DEFAULT_AUTH_DB_NAME)),
            key_path: read_path(&vars, "LAB_AUTH_KEY_PATH")
                .unwrap_or_else(|| default_auth_dir().join(DEFAULT_KEY_NAME)),
            bootstrap_secret: read_string(&vars, "LAB_AUTH_BOOTSTRAP_SECRET"),
            allowed_client_redirect_uris: read_csv(&vars, "LAB_AUTH_ALLOWED_REDIRECT_URIS")
                .unwrap_or_default(),
            allowed_emails: read_csv(&vars, "LAB_AUTH_ALLOWED_EMAILS")
                .unwrap_or_default()
                .into_iter()
                .map(|e| e.to_ascii_lowercase())
                .collect(),
            google: GoogleConfig {
                client_id: read_string(&vars, "LAB_GOOGLE_CLIENT_ID").unwrap_or_default(),
                client_secret: read_string(&vars, "LAB_GOOGLE_CLIENT_SECRET").unwrap_or_default(),
                callback_path: read_string(&vars, "LAB_GOOGLE_CALLBACK_PATH")
                    .unwrap_or_else(|| DEFAULT_CALLBACK_PATH.to_string()),
                scopes: read_csv(&vars, "LAB_GOOGLE_SCOPES").unwrap_or_else(default_google_scopes),
            },
            access_token_ttl: Duration::from_secs(
                read_u64(&vars, "LAB_AUTH_ACCESS_TOKEN_TTL_SECS")?
                    .unwrap_or(DEFAULT_ACCESS_TOKEN_TTL_SECS),
            ),
            refresh_token_ttl: Duration::from_secs(
                read_u64(&vars, "LAB_AUTH_REFRESH_TOKEN_TTL_SECS")?
                    .unwrap_or(DEFAULT_REFRESH_TOKEN_TTL_SECS),
            ),
            auth_code_ttl: Duration::from_secs(
                read_u64(&vars, "LAB_AUTH_CODE_TTL_SECS")?.unwrap_or(DEFAULT_AUTH_CODE_TTL_SECS),
            ),
            register_requests_per_minute: read_u32(&vars, "LAB_AUTH_REGISTER_REQUESTS_PER_MINUTE")?
                .unwrap_or(DEFAULT_REGISTER_REQUESTS_PER_MINUTE),
            authorize_requests_per_minute: read_u32(
                &vars,
                "LAB_AUTH_AUTHORIZE_REQUESTS_PER_MINUTE",
            )?
            .unwrap_or(DEFAULT_AUTHORIZE_REQUESTS_PER_MINUTE),
            max_pending_oauth_states: read_usize(&vars, "LAB_AUTH_MAX_PENDING_OAUTH_STATES")?
                .unwrap_or(DEFAULT_MAX_PENDING_OAUTH_STATES),
        };

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), AuthError> {
        if !self.google.callback_path.starts_with('/') {
            return Err(AuthError::Config(format!(
                "LAB_GOOGLE_CALLBACK_PATH must start with `/`, got `{}`",
                self.google.callback_path
            )));
        }

        if matches!(self.mode, AuthMode::OAuth) {
            if self.public_url.is_none() {
                return Err(AuthError::Config(
                    "LAB_PUBLIC_URL is required when LAB_AUTH_MODE=oauth".to_string(),
                ));
            }
            if self.google.client_id.is_empty() {
                return Err(AuthError::Config(
                    "LAB_GOOGLE_CLIENT_ID is required when LAB_AUTH_MODE=oauth".to_string(),
                ));
            }
            if self.google.client_secret.is_empty() {
                return Err(AuthError::Config(
                    "LAB_GOOGLE_CLIENT_SECRET is required when LAB_AUTH_MODE=oauth".to_string(),
                ));
            }
        }

        Ok(())
    }
}

fn normalize(vars: impl IntoIterator<Item = (String, String)>) -> HashMap<String, String> {
    vars.into_iter()
        .filter_map(|(key, value)| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some((key, trimmed.to_string()))
            }
        })
        .collect()
}

fn default_auth_dir() -> PathBuf {
    home_dir().map_or_else(|| PathBuf::from(".lab"), |home| home.join(".lab"))
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn default_callback_path() -> String {
    DEFAULT_CALLBACK_PATH.to_string()
}

fn default_google_scopes() -> Vec<String> {
    vec![
        "openid".to_string(),
        "email".to_string(),
        "profile".to_string(),
    ]
}

fn read_string(vars: &HashMap<String, String>, key: &str) -> Option<String> {
    vars.get(key).cloned()
}

fn read_path(vars: &HashMap<String, String>, key: &str) -> Option<PathBuf> {
    read_string(vars, key).map(PathBuf::from)
}

fn read_csv(vars: &HashMap<String, String>, key: &str) -> Option<Vec<String>> {
    read_string(vars, key).map(|value| {
        value
            .split(',')
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    })
}

fn read_url(vars: &HashMap<String, String>, key: &str) -> Result<Option<Url>, AuthError> {
    read_string(vars, key)
        .map(|value| {
            Url::parse(&value)
                .map_err(|error| AuthError::Config(format!("{key} must be a valid URL: {error}")))
        })
        .transpose()
}

fn read_u64(vars: &HashMap<String, String>, key: &str) -> Result<Option<u64>, AuthError> {
    read_string(vars, key)
        .map(|value| {
            value.parse::<u64>().map(Some).map_err(|error| {
                AuthError::Config(format!(
                    "{key} must be an integer number of seconds: {error}"
                ))
            })
        })
        .transpose()
        .map(Option::flatten)
}

fn read_u32(vars: &HashMap<String, String>, key: &str) -> Result<Option<u32>, AuthError> {
    read_string(vars, key)
        .map(|value| {
            value.parse::<u32>().map(Some).map_err(|error| {
                AuthError::Config(format!(
                    "{key} must be an integer number of requests per minute: {error}"
                ))
            })
        })
        .transpose()
        .map(|value| value.flatten())
}

fn read_usize(vars: &HashMap<String, String>, key: &str) -> Result<Option<usize>, AuthError> {
    read_string(vars, key)
        .map(|value| {
            value.parse::<usize>().map(Some).map_err(|error| {
                AuthError::Config(format!("{key} must be a positive integer: {error}"))
            })
        })
        .transpose()
        .map(|value| value.flatten())
}

#[cfg(test)]
mod tests {
    use super::{AuthConfig, AuthMode, AuthModeConfig};

    #[test]
    fn bearer_mode_preserves_existing_http_token_behavior() {
        let cfg = AuthModeConfig::from_sources(fake_env_with("LAB_AUTH_MODE", "bearer")).unwrap();
        assert!(matches!(cfg.mode, AuthMode::Bearer));
    }

    #[test]
    fn oauth_mode_requires_public_url_and_google_credentials() {
        let err = AuthConfig::from_sources(fake_env_with_many([
            ("LAB_AUTH_MODE", "oauth"),
            ("LAB_GOOGLE_CLIENT_ID", "id"),
        ]))
        .unwrap_err();
        assert!(err.to_string().contains("LAB_PUBLIC_URL"));
    }

    #[test]
    fn oauth_mode_defaults_paths_and_callback() {
        let cfg = AuthConfig::from_sources(fake_env_with_many([
            ("LAB_AUTH_MODE", "oauth"),
            ("LAB_PUBLIC_URL", "https://lab.example.com"),
            ("LAB_GOOGLE_CLIENT_ID", "id"),
            ("LAB_GOOGLE_CLIENT_SECRET", "secret"),
        ]))
        .unwrap();
        assert_eq!(cfg.sqlite_path.file_name().unwrap(), "auth.db");
        assert_eq!(cfg.key_path.file_name().unwrap(), "auth-jwt.pem");
        assert_eq!(cfg.google.callback_path, "/auth/google/callback");
    }

    #[test]
    fn allowed_emails_defaults_to_empty_vec() {
        let cfg = AuthConfig::from_sources(fake_env_with_many([
            ("LAB_AUTH_MODE", "oauth"),
            ("LAB_PUBLIC_URL", "https://lab.example.com"),
            ("LAB_GOOGLE_CLIENT_ID", "id"),
            ("LAB_GOOGLE_CLIENT_SECRET", "secret"),
        ]))
        .unwrap();
        assert!(cfg.allowed_emails.is_empty());
    }

    #[test]
    fn allowed_emails_normalizes_case_and_trims_whitespace() {
        let cfg = AuthConfig::from_sources(fake_env_with_many([
            ("LAB_AUTH_MODE", "oauth"),
            ("LAB_PUBLIC_URL", "https://lab.example.com"),
            ("LAB_GOOGLE_CLIENT_ID", "id"),
            ("LAB_GOOGLE_CLIENT_SECRET", "secret"),
            (
                "LAB_AUTH_ALLOWED_EMAILS",
                "Alice@Example.com , BOB@EXAMPLE.COM ",
            ),
        ]))
        .unwrap();
        assert_eq!(
            cfg.allowed_emails,
            vec![
                "alice@example.com".to_string(),
                "bob@example.com".to_string()
            ]
        );
    }

    #[test]
    fn oauth_mode_parses_allowed_client_redirect_uris() {
        let cfg = AuthConfig::from_sources(fake_env_with_many([
            ("LAB_AUTH_MODE", "oauth"),
            ("LAB_PUBLIC_URL", "https://lab.example.com"),
            ("LAB_GOOGLE_CLIENT_ID", "id"),
            ("LAB_GOOGLE_CLIENT_SECRET", "secret"),
            (
                "LAB_AUTH_ALLOWED_REDIRECT_URIS",
                "https://callback.tootie.tv/callback/*,https://claude.ai/api/mcp/auth_callback",
            ),
        ]))
        .unwrap();
        assert_eq!(
            cfg.allowed_client_redirect_uris,
            vec![
                "https://callback.tootie.tv/callback/*".to_string(),
                "https://claude.ai/api/mcp/auth_callback".to_string()
            ]
        );
    }

    fn fake_env_with(key: &'static str, value: &'static str) -> Vec<(String, String)> {
        vec![(key.to_string(), value.to_string())]
    }

    fn fake_env_with_many<const N: usize>(
        pairs: [(&'static str, &'static str); N],
    ) -> Vec<(String, String)> {
        pairs
            .into_iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }
}
