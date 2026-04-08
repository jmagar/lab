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
    /// # Errors
    /// Returns [`ApiError::Internal`] if the TLS backend fails to initialise
    /// (e.g. missing system crypto provider with rustls).
    pub fn new(base_url: impl Into<String>, auth: Auth) -> Result<Self, ApiError> {
        let inner = Client::builder()
            .user_agent(concat!("lab-apis/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| ApiError::Internal(format!("reqwest::Client::build: {e}")))?;
        Ok(Self {
            base_url: base_url.into(),
            auth,
            inner,
        })
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
        // Only relative paths are accepted. Absolute URLs would forward auth
        // headers to a foreign origin — explicitly rejected here.
        debug_assert!(
            !path.starts_with("http://") && !path.starts_with("https://"),
            "absolute URLs are not permitted; callers must pass relative paths"
        );
        if path.starts_with('/') {
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

    /// GET a path with query parameters and decode JSON.
    ///
    /// # Errors
    /// Returns [`ApiError`] on transport, status, or decode failure.
    pub async fn get_json_query<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<T, ApiError> {
        let mut url = reqwest::Url::parse(&self.url(path))
            .map_err(|e| ApiError::Internal(format!("invalid url: {e}")))?;
        if !query.is_empty() {
            {
                let mut pairs = url.query_pairs_mut();
                for (k, v) in query {
                    pairs.append_pair(k, v);
                }
            }
        }
        let resp = self
            .apply_auth(self.inner.get(url))
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

    /// PUT a JSON body and decode the JSON response.
    ///
    /// # Errors
    /// Returns [`ApiError`] on transport, status, or decode failure.
    pub async fn put_json<B: serde::Serialize + Sync, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let url = self.url(path);
        let resp = self
            .apply_auth(self.inner.put(&url).json(body))
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        Self::decode(resp).await
    }

    /// PATCH a JSON body and decode the JSON response.
    ///
    /// # Errors
    /// Returns [`ApiError`] on transport, status, or decode failure.
    pub async fn patch_json<B: serde::Serialize + Sync, T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let url = self.url(path);
        let resp = self
            .apply_auth(self.inner.patch(&url).json(body))
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        Self::decode(resp).await
    }

    /// GET a path, discarding the response body on success.
    ///
    /// # Errors
    /// Returns [`ApiError`] on transport or status failure.
    pub async fn get_void(&self, path: &str) -> Result<(), ApiError> {
        let url = self.url(path);
        let resp = self
            .apply_auth(self.inner.get(&url))
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        Self::check_status(resp).await
    }

    /// DELETE a path, discarding the response body on success.
    ///
    /// # Errors
    /// Returns [`ApiError`] on transport, status, or decode failure.
    pub async fn delete(&self, path: &str) -> Result<(), ApiError> {
        let url = self.url(path);
        let resp = self
            .apply_auth(self.inner.delete(&url))
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        Self::check_status(resp).await
    }

    /// DELETE a path with query parameters.
    ///
    /// # Errors
    /// Returns [`ApiError`] on transport or status failure.
    pub async fn delete_query(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<(), ApiError> {
        let mut url = reqwest::Url::parse(&self.url(path))
            .map_err(|e| ApiError::Internal(format!("invalid url: {e}")))?;
        if !query.is_empty() {
            {
                let mut pairs = url.query_pairs_mut();
                for (k, v) in query {
                    pairs.append_pair(k, v);
                }
            }
        }
        let resp = self
            .apply_auth(self.inner.delete(url))
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        Self::check_status(resp).await
    }

    /// POST a JSON body, discarding the response body on success.
    ///
    /// # Errors
    /// Returns [`ApiError`] on transport, status, or decode failure.
    pub async fn post_void<B: serde::Serialize + Sync>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<(), ApiError> {
        let url = self.url(path);
        let resp = self
            .apply_auth(self.inner.post(&url).json(body))
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        Self::check_status(resp).await
    }

    async fn check_status(resp: reqwest::Response) -> Result<(), ApiError> {
        if resp.status().is_success() {
            return Ok(());
        }
        let code = resp.status().as_u16();
        let body = resp
            .text()
            .await
            .unwrap_or_else(|e| format!("<failed to read response body: {e}>"));
        Err(match code {
            401 | 403 => ApiError::Auth,
            404 => ApiError::NotFound,
            429 => ApiError::RateLimited { retry_after: None },
            _ => ApiError::Server { status: code, body },
        })
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
        let body = resp
            .text()
            .await
            .unwrap_or_else(|e| format!("<failed to read response body: {e}>"));
        Err(match code {
            401 | 403 => ApiError::Auth,
            404 => ApiError::NotFound,
            429 => ApiError::RateLimited { retry_after: None },
            _ => ApiError::Server { status: code, body },
        })
    }
}
