//! `AppriseClient` — async methods against an apprise-api server.
//!
//! Endpoints land incrementally from `docs/upstream-api/apprise.md`.

use std::fmt::Write as _;

use crate::core::{Auth, HttpClient};

use super::error::AppriseError;

/// Client for an apprise-api notification dispatcher.
pub struct AppriseClient {
    #[allow(dead_code)]
    http: HttpClient,
}

impl AppriseClient {
    /// Build a client against `base_url`.
    ///
    /// apprise-api is typically unauthenticated (pass `Auth::None`) but can
    /// be fronted by a reverse proxy that injects basic auth or a bearer
    /// token — use `Auth::Basic { .. }` or `Auth::Bearer { .. }` accordingly.
    ///
    /// # Errors
    /// Returns [`AppriseError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, AppriseError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    /// Health check — `GET /status`.
    ///
    /// # Errors
    /// Returns `AppriseError::Api` on HTTP failure.
    pub async fn health(&self) -> Result<(), AppriseError> {
        self.http.get_void("/status").await?;
        Ok(())
    }

    /// Send a stateless notification to URLs supplied in the request body.
    ///
    /// # Errors
    /// Returns `AppriseError::Api` on HTTP failure.
    pub async fn notify(&self, request: &super::types::NotifyRequest) -> Result<(), AppriseError> {
        self.http.post_void("/notify", request).await?;
        Ok(())
    }

    /// Send a notification to the stored config identified by `key`.
    ///
    /// # Errors
    /// Returns `AppriseError::Api` on HTTP failure.
    pub async fn notify_key(
        &self,
        key: &str,
        request: &super::types::NotifyRequest,
    ) -> Result<(), AppriseError> {
        let path = format!("/notify/{}", encode_path_segment(key));
        self.http.post_void(&path, request).await?;
        Ok(())
    }
}

/// Percent-encode a URL path segment (RFC 3986 unreserved chars pass through unchanged).
fn encode_path_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') {
            out.push(c);
        } else {
            let mut buf = [0u8; 4];
            let encoded = c.encode_utf8(&mut buf);
            for byte in encoded.bytes() {
                let _ = write!(out, "%{byte:02X}");
            }
        }
    }
    out
}
