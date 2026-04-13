//! Shared HTTP client — thin reqwest wrapper with auth injection and JSON helpers.

use std::time::{Duration, Instant};

use reqwest::{Client, RequestBuilder, Response, Url};
use tracing::{Level, event};

use crate::core::auth::Auth;
use crate::core::error::ApiError;

// ---------------------------------------------------------------------------
// Private GraphQL envelope types
// ---------------------------------------------------------------------------

#[derive(serde::Serialize)]
struct GraphQlRequest<'a> {
    query: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<&'a serde_json::Value>,
}

#[derive(serde::Deserialize)]
struct GraphQlResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQlError>>,
}

#[derive(serde::Deserialize)]
struct GraphQlError {
    message: String,
}

// ---------------------------------------------------------------------------

/// Shared HTTP client. Cheap to clone — wraps `reqwest::Client` which is `Arc`-based internally.
#[derive(Debug, Clone)]
pub struct HttpClient {
    base_url: String,
    auth: Auth,
    inner: Client,
}

struct RequestLogContext {
    method: &'static str,
    path: String,
    host: String,
    start: Instant,
}

impl RequestLogContext {
    fn new(method: &'static str, url: &Url) -> Self {
        Self {
            method,
            path: url.path().to_string(),
            host: url.host_str().unwrap_or_default().to_string(),
            start: Instant::now(),
        }
    }

    fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }
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
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
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

    fn url(&self, path: &str) -> Result<String, ApiError> {
        // Only relative paths are accepted. Absolute URLs would forward auth
        // headers to a foreign origin — rejected at runtime in all build profiles.
        if path.starts_with("http://") || path.starts_with("https://") {
            return Err(ApiError::Internal(format!(
                "absolute URL not permitted: {path}"
            )));
        }
        if path.starts_with('/') {
            Ok(format!("{}{path}", self.base_url.trim_end_matches('/')))
        } else {
            Ok(format!("{}/{path}", self.base_url.trim_end_matches('/')))
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
        let url = Url::parse(&self.url(path)?)
            .map_err(|e| ApiError::Internal(format!("invalid url: {e}")))?;
        let ctx = RequestLogContext::new("GET", &url);
        let resp = self
            .send(self.apply_auth(self.inner.get(url.clone())), &ctx)
            .await?;
        Self::decode(resp, &ctx).await
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
        let mut url = Url::parse(&self.url(path)?)
            .map_err(|e| ApiError::Internal(format!("invalid url: {e}")))?;
        if !query.is_empty() {
            {
                let mut pairs = url.query_pairs_mut();
                for (k, v) in query {
                    pairs.append_pair(k, v);
                }
            }
        }
        let ctx = RequestLogContext::new("GET", &url);
        let resp = self
            .send(self.apply_auth(self.inner.get(url.clone())), &ctx)
            .await?;
        Self::decode(resp, &ctx).await
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
        let url = Url::parse(&self.url(path)?)
            .map_err(|e| ApiError::Internal(format!("invalid url: {e}")))?;
        let ctx = RequestLogContext::new("POST", &url);
        let resp = self
            .send(
                self.apply_auth(self.inner.post(url.clone()).json(body)),
                &ctx,
            )
            .await?;
        Self::decode(resp, &ctx).await
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
        let url = Url::parse(&self.url(path)?)
            .map_err(|e| ApiError::Internal(format!("invalid url: {e}")))?;
        let ctx = RequestLogContext::new("PUT", &url);
        let resp = self
            .send(
                self.apply_auth(self.inner.put(url.clone()).json(body)),
                &ctx,
            )
            .await?;
        Self::decode(resp, &ctx).await
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
        let url = Url::parse(&self.url(path)?)
            .map_err(|e| ApiError::Internal(format!("invalid url: {e}")))?;
        let ctx = RequestLogContext::new("PATCH", &url);
        let resp = self
            .send(
                self.apply_auth(self.inner.patch(url.clone()).json(body)),
                &ctx,
            )
            .await?;
        Self::decode(resp, &ctx).await
    }

    /// GET a path, discarding the response body on success.
    ///
    /// # Errors
    /// Returns [`ApiError`] on transport or status failure.
    pub async fn get_void(&self, path: &str) -> Result<(), ApiError> {
        let url = Url::parse(&self.url(path)?)
            .map_err(|e| ApiError::Internal(format!("invalid url: {e}")))?;
        let ctx = RequestLogContext::new("GET", &url);
        let resp = self
            .send(self.apply_auth(self.inner.get(url.clone())), &ctx)
            .await?;
        Self::check_status(resp, &ctx).await
    }

    /// DELETE a path, discarding the response body on success.
    ///
    /// # Errors
    /// Returns [`ApiError`] on transport, status, or decode failure.
    pub async fn delete(&self, path: &str) -> Result<(), ApiError> {
        let url = Url::parse(&self.url(path)?)
            .map_err(|e| ApiError::Internal(format!("invalid url: {e}")))?;
        let ctx = RequestLogContext::new("DELETE", &url);
        let resp = self
            .send(self.apply_auth(self.inner.delete(url.clone())), &ctx)
            .await?;
        Self::check_status(resp, &ctx).await
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
        let mut url = Url::parse(&self.url(path)?)
            .map_err(|e| ApiError::Internal(format!("invalid url: {e}")))?;
        if !query.is_empty() {
            {
                let mut pairs = url.query_pairs_mut();
                for (k, v) in query {
                    pairs.append_pair(k, v);
                }
            }
        }
        let ctx = RequestLogContext::new("DELETE", &url);
        let resp = self
            .send(self.apply_auth(self.inner.delete(url.clone())), &ctx)
            .await?;
        Self::check_status(resp, &ctx).await
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
        let url = Url::parse(&self.url(path)?)
            .map_err(|e| ApiError::Internal(format!("invalid url: {e}")))?;
        let ctx = RequestLogContext::new("POST", &url);
        let resp = self
            .send(
                self.apply_auth(self.inner.post(url.clone()).json(body)),
                &ctx,
            )
            .await?;
        Self::check_status(resp, &ctx).await
    }

    /// POST a GraphQL query and decode the `data` field of the response.
    ///
    /// Unlike REST endpoints, GraphQL servers always return HTTP 200 — even when the
    /// operation fails. Errors are conveyed in a top-level `errors[]` array alongside
    /// (or instead of) `data`. This method handles that contract:
    ///
    /// - Sends `{"query": ..., "variables": {...}}` as a JSON body.
    /// - If `errors[]` is present, all error messages are joined with `"; "` and
    ///   returned as `ApiError::Server { status: 200, body: <joined> }`. Errors take
    ///   priority — if both `data` and `errors` are present, the error is returned.
    /// - If `errors` is absent but `data` is `null` or missing, returns
    ///   `ApiError::Decode("GraphQL response missing data field")`.
    /// - On success, deserialises `data` directly into `T` (the caller provides the
    ///   wrapper type matching the query's selection set).
    ///
    /// # Errors
    /// Returns [`ApiError`] on transport, status (from the underlying HTTP layer),
    /// GraphQL application errors, or JSON decode failure.
    pub async fn post_graphql<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &str,
        variables: Option<&serde_json::Value>,
    ) -> Result<T, ApiError> {
        let body = GraphQlRequest { query, variables };
        let resp: GraphQlResponse<T> = self.post_json(path, &body).await?;
        if let Some(errors) = resp.errors {
            let msg = errors
                .iter()
                .map(|e| e.message.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(ApiError::Server {
                status: 200,
                body: msg,
            });
        }
        resp.data
            .ok_or_else(|| ApiError::Decode("GraphQL response missing data field".into()))
    }

    /// Map a non-success HTTP status code and response body into an [`ApiError`].
    fn error_for_status(code: u16, body: String) -> ApiError {
        match code {
            401 | 403 => ApiError::Auth,
            404 => ApiError::NotFound,
            429 => ApiError::RateLimited { retry_after: None },
            _ => ApiError::Server { status: code, body },
        }
    }

    /// Read the response body as text, preserving read errors.
    async fn read_error_body(resp: Response) -> (u16, String) {
        let code = resp.status().as_u16();
        let body = resp
            .text()
            .await
            .unwrap_or_else(|e| format!("<failed to read response body: {e}>"));
        (code, body)
    }

    async fn check_status(resp: Response, ctx: &RequestLogContext) -> Result<(), ApiError> {
        if resp.status().is_success() {
            Self::log_finish(ctx, resp.status().as_u16());
            return Ok(());
        }
        let (code, body) = Self::read_error_body(resp).await;
        let err = Self::error_for_status(code, body);
        Self::log_error(ctx, &err);
        Err(err)
    }

    async fn decode<T: serde::de::DeserializeOwned>(
        resp: Response,
        ctx: &RequestLogContext,
    ) -> Result<T, ApiError> {
        if resp.status().is_success() {
            let status = resp.status().as_u16();
            let decoded = resp
                .json::<T>()
                .await
                .map_err(|e| ApiError::Decode(e.to_string()));
            match &decoded {
                Ok(_) => Self::log_finish(ctx, status),
                Err(err) => Self::log_error(ctx, err),
            }
            return decoded;
        }
        let (code, body) = Self::read_error_body(resp).await;
        let err = Self::error_for_status(code, body);
        Self::log_error(ctx, &err);
        Err(err)
    }

    async fn send(
        &self,
        request: RequestBuilder,
        ctx: &RequestLogContext,
    ) -> Result<Response, ApiError> {
        event!(
            Level::INFO,
            method = ctx.method,
            path = ctx.path.as_str(),
            host = ctx.host.as_str(),
            "request.start"
        );
        request.send().await.map_err(|e| {
            let err = ApiError::Network(e.to_string());
            Self::log_error(ctx, &err);
            err
        })
    }

    fn log_finish(ctx: &RequestLogContext, status: u16) {
        event!(
            Level::INFO,
            method = ctx.method,
            path = ctx.path.as_str(),
            host = ctx.host.as_str(),
            status,
            elapsed_ms = ctx.elapsed_ms(),
            "request.finish"
        );
    }

    fn log_error(ctx: &RequestLogContext, err: &ApiError) {
        let status: Option<u16> = match err {
            ApiError::Auth => Some(401),
            ApiError::NotFound => Some(404),
            ApiError::RateLimited { .. } => Some(429),
            ApiError::Server { status, .. } => Some(*status),
            _ => None,
        };
        match err {
            ApiError::Internal(_) => event!(
                Level::ERROR,
                method = ctx.method,
                path = ctx.path.as_str(),
                host = ctx.host.as_str(),
                elapsed_ms = ctx.elapsed_ms(),
                status,
                kind = err.kind(),
                message = %err,
                "request.error"
            ),
            ApiError::Auth
            | ApiError::NotFound
            | ApiError::RateLimited { .. }
            | ApiError::Validation { .. }
            | ApiError::Network(_)
            | ApiError::Server { .. }
            | ApiError::Decode(_) => event!(
                Level::WARN,
                method = ctx.method,
                path = ctx.path.as_str(),
                host = ctx.host.as_str(),
                elapsed_ms = ctx.elapsed_ms(),
                status,
                kind = err.kind(),
                message = %err,
                "request.error"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::auth::Auth;

    fn make_client(base_url: &str) -> HttpClient {
        HttpClient::new(base_url, Auth::None).expect("client construction should succeed")
    }

    #[test]
    fn absolute_url_rejected_at_runtime() {
        let client = make_client("http://localhost:8080");

        let err_http = client.url("http://evil.example.com/steal");
        assert!(
            matches!(err_http, Err(ApiError::Internal(ref msg)) if msg.contains("absolute URL not permitted")),
            "expected Internal error for http:// path, got: {err_http:?}"
        );

        let err_https = client.url("https://evil.example.com/steal");
        assert!(
            matches!(err_https, Err(ApiError::Internal(ref msg)) if msg.contains("absolute URL not permitted")),
            "expected Internal error for https:// path, got: {err_https:?}"
        );
    }

    #[test]
    fn relative_paths_accepted() {
        let client = make_client("http://localhost:8080");

        let url = client
            .url("/api/v1/status")
            .expect("relative path should be accepted");
        assert_eq!(url, "http://localhost:8080/api/v1/status");

        let url2 = client
            .url("api/v1/status")
            .expect("bare relative path should be accepted");
        assert_eq!(url2, "http://localhost:8080/api/v1/status");
    }

    #[test]
    fn base_url_trailing_slash_normalised() {
        let client = make_client("http://localhost:8080/");

        let url = client
            .url("/api/v1/status")
            .expect("should normalise trailing slash");
        assert_eq!(url, "http://localhost:8080/api/v1/status");
    }
}
