//! Shared HTTP client — thin reqwest wrapper with auth injection and JSON helpers.

use reqwest::{Client, RequestBuilder};

use crate::core::auth::Auth;
use crate::core::error::ApiError;

/// Shared HTTP client. Cheap to clone — wraps `reqwest::Client` which is `Arc`-based internally.
#[derive(Debug, Clone)]
pub struct HttpClient {
    base_url: String,
    auth: Auth,
    inner: Client,
}

impl HttpClient {
    /// Construct a new client with a base URL and auth strategy.
    ///
    /// # Panics
    /// Panics if the system TLS backend cannot initialize. This only happens
    /// in environments without a working rustls / system crypto provider,
    /// which would make every subsequent request fail anyway — panicking here
    /// surfaces the misconfiguration at startup instead of on first call.
    #[allow(clippy::expect_used)] // startup TLS init — see # Panics doc
    #[must_use]
    pub fn new(base_url: impl Into<String>, auth: Auth) -> Self {
        let inner = Client::builder()
            .user_agent(concat!("lab-apis/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("reqwest::Client::build (rustls TLS backend must initialize)");
        Self {
            base_url: base_url.into(),
            auth,
            inner,
        }
    }

    /// Base URL this client targets.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Auth strategy.
    #[must_use]
    pub const fn auth(&self) -> &Auth {
        &self.auth
    }

    fn url(&self, path: &str) -> String {
        if path.starts_with("http://") || path.starts_with("https://") {
            path.to_string()
        } else if path.starts_with('/') {
            format!("{}{path}", self.base_url.trim_end_matches('/'))
        } else {
            format!("{}/{path}", self.base_url.trim_end_matches('/'))
        }
    }

    fn apply_auth(&self, req: RequestBuilder) -> RequestBuilder {
        match &self.auth {
            Auth::None => req,
            Auth::ApiKey { header, key } => req.header(header, key),
            Auth::Token { token } => req.header("Authorization", format!("Token {token}")),
            Auth::Bearer { token } => req.bearer_auth(token),
            Auth::Basic { username, password } => req.basic_auth(username, Some(password)),
            Auth::Session { cookie } => req.header("Cookie", cookie),
        }
    }

    /// GET a path and decode JSON.
    ///
    /// # Errors
    /// Returns [`ApiError`] on transport, status, or decode failure.
    pub async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T, ApiError> {
        let url = self.url(path);
        let resp = self
            .apply_auth(self.inner.get(&url))
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        Self::decode(resp).await
    }

    /// POST a JSON body and decode the JSON response.
    ///
    /// # Errors
    /// Returns [`ApiError`] on transport, status, or decode failure.
    pub async fn post_json<B: serde::Serialize + Sync, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let url = self.url(path);
        let resp = self
            .apply_auth(self.inner.post(&url).json(body))
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        Self::decode(resp).await
    }

    async fn decode<T: serde::de::DeserializeOwned>(
        resp: reqwest::Response,
    ) -> Result<T, ApiError> {
        let status = resp.status();
        if status.is_success() {
            return resp
                .json::<T>()
                .await
                .map_err(|e| ApiError::Decode(e.to_string()));
        }
        let code = status.as_u16();
        let body = resp.text().await.unwrap_or_default();
        Err(match code {
            401 | 403 => ApiError::Auth,
            404 => ApiError::NotFound,
            429 => ApiError::RateLimited { retry_after: None },
            _ => ApiError::Server { status: code, body },
        })
    }
}
