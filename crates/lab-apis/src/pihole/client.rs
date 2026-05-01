//! Pi-hole v6 API client.

use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::{Value, json};
use tokio::sync::Mutex;

use crate::core::error::ApiError;

use super::error::PiholeError;
use super::types::{PiholeResponse, QueryLogResponse};

const SESSION_GRACE: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
struct PiholeSession {
    sid: String,
    csrf: Option<String>,
    expires_at: Instant,
}

impl PiholeSession {
    fn is_fresh(&self) -> bool {
        Instant::now() < self.expires_at
    }

    fn headers(&self) -> Result<HeaderMap, PiholeError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-FTL-SID",
            HeaderValue::from_str(&self.sid)
                .map_err(|e| ApiError::Internal(format!("invalid Pi-hole sid header: {e}")))?,
        );
        if let Some(csrf) = &self.csrf {
            headers.insert(
                "X-FTL-CSRF",
                HeaderValue::from_str(csrf)
                    .map_err(|e| ApiError::Internal(format!("invalid Pi-hole csrf header: {e}")))?,
            );
        }
        Ok(headers)
    }
}

/// Client for a Pi-hole v6 instance.
#[derive(Clone)]
pub struct PiholeClient {
    base_url: String,
    password: String,
    totp: Option<String>,
    inner: reqwest::Client,
    session: Arc<Mutex<Option<PiholeSession>>>,
}

impl PiholeClient {
    /// Build a Pi-hole v6 client.
    pub fn new(
        base_url: &str,
        password: String,
        totp: Option<String>,
    ) -> Result<Self, PiholeError> {
        if password.is_empty() {
            return Err(PiholeError::InvalidParam(
                "PIHOLE_PASSWORD must not be empty".into(),
            ));
        }
        let inner = reqwest::Client::builder()
            .user_agent(concat!("lab-apis/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| ApiError::Internal(format!("reqwest::Client::build: {e}")))?;
        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            password,
            totp,
            inner,
            session: Arc::new(Mutex::new(None)),
        })
    }

    /// Fetch summary counters.
    pub async fn summary(&self) -> Result<PiholeResponse, PiholeError> {
        self.get_value("/summaries", &[]).await
    }

    /// Fetch current settings.
    pub async fn settings(&self) -> Result<PiholeResponse, PiholeError> {
        self.get_value("/settings", &[]).await
    }

    /// Fetch DNS blocking state.
    pub async fn blocking_status(&self) -> Result<PiholeResponse, PiholeError> {
        self.get_value("/dns/blocking", &[]).await
    }

    /// Set DNS blocking state.
    pub async fn blocking_set(
        &self,
        blocking: bool,
        timer_seconds: Option<u32>,
    ) -> Result<PiholeResponse, PiholeError> {
        let body = if let Some(seconds) = timer_seconds {
            json!({ "blocking": blocking, "timer": seconds })
        } else {
            json!({ "blocking": blocking })
        };
        self.post_value("/dns/blocking", &body).await
    }

    /// Search bounded query logs.
    pub async fn querylog_search(
        &self,
        offset: u32,
        limit: u32,
    ) -> Result<QueryLogResponse, PiholeError> {
        if !(1..=500).contains(&limit) {
            return Err(PiholeError::InvalidParam(
                "querylog limit must be between 1 and 500".into(),
            ));
        }
        let value = self
            .get_json(
                "/logs",
                &[
                    ("offset".to_string(), offset.to_string()),
                    ("limit".to_string(), limit.to_string()),
                ],
                true,
            )
            .await?;
        Ok(QueryLogResponse {
            value: redact_querylog(value),
            offset,
            limit,
        })
    }

    /// List adlists.
    pub async fn adlist_list(&self) -> Result<PiholeResponse, PiholeError> {
        self.get_value("/adlists", &[]).await
    }

    /// Add an adlist URL.
    pub async fn adlist_add(&self, address: &str) -> Result<PiholeResponse, PiholeError> {
        if address.trim().is_empty() {
            return Err(PiholeError::InvalidParam(
                "adlist address must not be empty".into(),
            ));
        }
        self.post_value("/adlists", &json!({ "address": address }))
            .await
    }

    /// Remove an adlist by id.
    pub async fn adlist_remove(&self, id: &str) -> Result<PiholeResponse, PiholeError> {
        let id = encode_path_segment(id)?;
        self.delete_value(&format!("/adlists/{id}")).await
    }

    /// List domain rules.
    pub async fn domain_list(&self) -> Result<PiholeResponse, PiholeError> {
        self.get_value("/domains", &[]).await
    }

    /// Add a domain rule.
    pub async fn domain_add(
        &self,
        domain: &str,
        domain_type: u8,
        comment: Option<&str>,
    ) -> Result<PiholeResponse, PiholeError> {
        if domain.trim().is_empty() {
            return Err(PiholeError::InvalidParam("domain must not be empty".into()));
        }
        if domain_type > 3 {
            return Err(PiholeError::InvalidParam(
                "domain_type must be 0, 1, 2, or 3".into(),
            ));
        }
        let mut body = json!({ "domain": domain, "type": domain_type });
        if let Some(comment) = comment {
            body["comment"] = Value::String(comment.to_string());
        }
        self.post_value("/domains", &body).await
    }

    async fn get_value(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<PiholeResponse, PiholeError> {
        Ok(PiholeResponse {
            value: self.get_json(path, query, true).await?,
        })
    }

    async fn post_value(&self, path: &str, body: &Value) -> Result<PiholeResponse, PiholeError> {
        Ok(PiholeResponse {
            value: self
                .send_json(reqwest::Method::POST, path, &[], Some(body), true)
                .await?,
        })
    }

    async fn delete_value(&self, path: &str) -> Result<PiholeResponse, PiholeError> {
        Ok(PiholeResponse {
            value: self
                .send_json(reqwest::Method::DELETE, path, &[], None, true)
                .await?,
        })
    }

    async fn get_json(
        &self,
        path: &str,
        query: &[(String, String)],
        retry_auth: bool,
    ) -> Result<Value, PiholeError> {
        self.send_json(reqwest::Method::GET, path, query, None, retry_auth)
            .await
    }

    async fn send_json(
        &self,
        method: reqwest::Method,
        path: &str,
        query: &[(String, String)],
        body: Option<&Value>,
        retry_auth: bool,
    ) -> Result<Value, PiholeError> {
        let headers = self.ensure_session().await?.headers()?;
        let mut url = self.url(path)?;
        if !query.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in query {
                pairs.append_pair(key, value);
            }
        }
        let mut request = self.inner.request(method.clone(), url);
        request = request.headers(headers);
        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        if response.status() == reqwest::StatusCode::UNAUTHORIZED && retry_auth {
            self.clear_session().await;
            return Box::pin(self.send_json(method, path, query, body, false)).await;
        }
        Self::decode_response(response).await
    }

    async fn ensure_session(&self) -> Result<PiholeSession, PiholeError> {
        let mut guard = self.session.lock().await;
        if let Some(session) = guard.as_ref().filter(|session| session.is_fresh()) {
            return Ok(session.clone());
        }
        let session = self.login().await?;
        *guard = Some(session.clone());
        Ok(session)
    }

    async fn clear_session(&self) {
        *self.session.lock().await = None;
    }

    async fn login(&self) -> Result<PiholeSession, PiholeError> {
        let mut body = json!({ "password": self.password });
        if let Some(totp) = &self.totp {
            body["totp"] = Value::String(totp.clone());
        }
        let response = self
            .inner
            .post(self.url("/auth")?)
            .json(&body)
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        let value = Self::decode_response(response).await?;
        let session = value
            .get("session")
            .ok_or_else(|| ApiError::Decode("missing session object".into()))?;
        let sid = session
            .get("sid")
            .and_then(Value::as_str)
            .ok_or_else(|| ApiError::Decode("missing session.sid".into()))?
            .to_string();
        let csrf = session
            .get("csrf")
            .and_then(Value::as_str)
            .map(str::to_string);
        let validity = session
            .get("validity")
            .and_then(Value::as_u64)
            .unwrap_or(300);
        let ttl = Duration::from_secs(validity).saturating_sub(SESSION_GRACE);
        Ok(PiholeSession {
            sid,
            csrf,
            expires_at: Instant::now() + ttl,
        })
    }

    fn url(&self, path: &str) -> Result<reqwest::Url, PiholeError> {
        if path.contains("?auth=") {
            return Err(PiholeError::InvalidParam(
                "Pi-hole v5 ?auth= token query is not supported".into(),
            ));
        }
        let base = if self.base_url.ends_with("/api") {
            self.base_url.clone()
        } else {
            format!("{}/api", self.base_url.trim_end_matches('/'))
        };
        let path = path.trim_start_matches('/');
        reqwest::Url::parse(&format!("{base}/{path}"))
            .map_err(|e| ApiError::Internal(format!("invalid Pi-hole URL: {e}")).into())
    }

    async fn decode_response(response: reqwest::Response) -> Result<Value, PiholeError> {
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        if status.is_success() {
            if text.trim().is_empty() {
                return Ok(Value::Null);
            }
            return serde_json::from_str(&text).map_err(|e| ApiError::Decode(e.to_string()).into());
        }

        let message = pihole_error_message(&text).unwrap_or_else(|| text.clone());
        let api = match status {
            reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => ApiError::Auth,
            reqwest::StatusCode::NOT_FOUND => ApiError::NotFound,
            reqwest::StatusCode::TOO_MANY_REQUESTS => ApiError::RateLimited { retry_after: None },
            status if status.is_client_error() => ApiError::Validation {
                field: "pihole".into(),
                message,
            },
            _ => ApiError::Server {
                status: status.as_u16(),
                body: message,
            },
        };
        Err(api.into())
    }
}

fn pihole_error_message(text: &str) -> Option<String> {
    let value = serde_json::from_str::<Value>(text).ok()?;
    let error = value.get("error")?;
    let message = error.get("message").and_then(Value::as_str).unwrap_or(text);
    let hint = error.get("hint").and_then(Value::as_str);
    Some(match hint {
        Some(hint) if !hint.is_empty() => format!("{message}: {hint}"),
        _ => message.to_string(),
    })
}

fn encode_path_segment(value: &str) -> Result<String, PiholeError> {
    if value.trim().is_empty() || value.contains('/') || value.contains('?') || value.contains('#')
    {
        return Err(PiholeError::InvalidParam(
            "path id must be a non-empty single segment".into(),
        ));
    }
    Ok(url::form_urlencoded::byte_serialize(value.as_bytes()).collect())
}

fn redact_querylog(mut value: Value) -> Value {
    fn strip(value: &mut Value) {
        match value {
            Value::Object(map) => {
                for key in [
                    "client",
                    "client_ip",
                    "client_name",
                    "mac",
                    "upstream",
                    "reply_addr",
                ] {
                    map.remove(key);
                }
                for value in map.values_mut() {
                    strip(value);
                }
            }
            Value::Array(values) => {
                for value in values {
                    strip(value);
                }
            }
            _ => {}
        }
    }
    strip(&mut value);
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_legacy_auth_query() {
        let client = PiholeClient::new("http://example.test", "pw".into(), None).unwrap();
        assert!(client.url("/logs?auth=legacy").is_err());
    }
}
