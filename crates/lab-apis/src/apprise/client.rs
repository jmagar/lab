//! `AppriseClient` — async methods against an apprise-api server.
//!
//! Endpoints land incrementally from `docs/upstream-api/apprise.md`.

use std::fmt::Write as _;

use crate::core::{Auth, HttpClient};

use super::error::AppriseError;
use super::types::UrlInfo;

/// Client for an apprise-api notification dispatcher.
pub struct AppriseClient {
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

    /// Persist a YAML/text config blob under `key` — `POST /add/{key}`.
    ///
    /// Sends a JSON body with `{"config": "<yaml>", "format": "yaml"}`.
    /// For plain URL lists pass `format = "text"`. The Apprise API accepts
    /// `Content-Type: application/json` for this endpoint.
    ///
    /// # Errors
    /// Returns `AppriseError::Api` on HTTP failure.
    pub async fn add_config(&self, key: &str, config: &str, format: &str) -> Result<(), AppriseError> {
        let path = format!("/add/{}", encode_path_segment(key));
        let body = super::types::AddRequest {
            urls: None,
            config: Some(config.to_owned()),
            format: Some(format.to_owned()),
        };
        self.http.post_void(&path, &body).await?;
        Ok(())
    }

    /// Retrieve the stored config for `key` — `POST /get/{key}`.
    ///
    /// Returns the raw YAML/text config blob. Apprise-api uses POST (not GET)
    /// for this endpoint to mirror its `del` counterpart.
    ///
    /// # Errors
    /// Returns `AppriseError::Api` on HTTP failure or if the key does not exist.
    pub async fn get_config(&self, key: &str) -> Result<String, AppriseError> {
        let path = format!("/get/{}", encode_path_segment(key));
        Ok(self.http.post_empty_get_text(&path).await?)
    }

    /// Delete the stored config for `key` — `POST /del/{key}`.
    ///
    /// # Errors
    /// Returns `AppriseError::Api` on HTTP failure.
    pub async fn delete_config(&self, key: &str) -> Result<(), AppriseError> {
        let path = format!("/del/{}", encode_path_segment(key));
        // Apprise-api /del/{key} accepts an empty POST body.
        self.http.post_text_void(&path, "").await?;
        Ok(())
    }

    /// List the notification URLs stored under `key` — `GET /json/urls/{key}`.
    ///
    /// Returns a list of `UrlInfo` objects, each with a `url` and optional `tags`.
    ///
    /// # Errors
    /// Returns `AppriseError::Api` on HTTP failure or if the key does not exist.
    pub async fn get_urls(&self, key: &str) -> Result<Vec<UrlInfo>, AppriseError> {
        let path = format!("/json/urls/{}", encode_path_segment(key));
        Ok(self.http.get_json(&path).await?)
    }

    /// Retrieve server details listing all loaded Apprise plugins — `GET /details`.
    ///
    /// Returns a large JSON blob whose schema varies by Apprise version. Callers
    /// should treat the response as an opaque `serde_json::Value`.
    ///
    /// # Errors
    /// Returns `AppriseError::Api` on HTTP failure.
    pub async fn details(&self) -> Result<serde_json::Value, AppriseError> {
        Ok(self.http.get_json("/details").await?)
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
