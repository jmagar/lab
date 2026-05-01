//! `DozzleClient` read-only API methods.

use std::time::{Duration, Instant};

use futures::StreamExt;
use reqwest::Url;
use serde_json::Value;

use crate::core::error::ApiError;
use crate::core::{Auth, HttpClient};

use super::error::DozzleError;
use super::types::{
    ContainersListResponse, HealthResponse, LimitMetadata, LimitedText, LogFetchRequest,
    LogFetchResponse, PlainLogFetchResponse, ReadLimits, VersionResponse,
};

/// Client for a Dozzle instance.
#[derive(Clone)]
pub struct DozzleClient {
    http: HttpClient,
    base_url: String,
    auth: Auth,
    inner: reqwest::Client,
}

impl DozzleClient {
    /// Build a client against `base_url` with the given auth.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, DozzleError> {
        let http = HttpClient::new(base_url, auth.clone())?;
        let inner = reqwest::Client::builder()
            .user_agent(concat!("lab-apis/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| ApiError::Internal(format!("reqwest::Client::build: {e}")))?;
        Ok(Self {
            http,
            base_url: base_url.to_string(),
            auth,
            inner,
        })
    }

    /// Probe `/healthcheck`.
    pub async fn health(&self) -> Result<HealthResponse, DozzleError> {
        self.http.get_void("/healthcheck").await?;
        Ok(HealthResponse { reachable: true })
    }

    /// Fetch and normalize `/api/version`.
    pub async fn version(&self) -> Result<VersionResponse, DozzleError> {
        let raw = self.http.get_text("/api/version").await?;
        let version = raw
            .trim()
            .strip_prefix("<pre>")
            .and_then(|s| s.strip_suffix("</pre>"))
            .unwrap_or(raw.trim())
            .trim()
            .to_string();
        Ok(VersionResponse { version })
    }

    /// Return the first container inventory emitted by `/api/events/stream`.
    pub async fn containers_list(
        &self,
        limits: ReadLimits,
    ) -> Result<ContainersListResponse, DozzleError> {
        let started = Instant::now();
        let limits = limits.capped();
        let url = self.url_with_query("/api/events/stream", &[])?;
        let events = self.read_sse_events(url, &limits).await?;
        let events_read = events.len();
        for event in events {
            if event.event == "containers-changed" {
                let containers = event.data.as_array().cloned().ok_or_else(|| {
                    DozzleError::InvalidResponse(
                        "containers-changed event data was not an array".into(),
                    )
                })?;
                return Ok(ContainersListResponse {
                    containers,
                    meta: LimitMetadata {
                        truncated: false,
                        limit_kind: None,
                        bytes_read: event.bytes_read,
                        events_read: Some(events_read),
                        lines_read: None,
                        duration_ms: started.elapsed().as_millis(),
                    },
                });
            }
        }
        Err(DozzleError::StreamTimeout(
            "containers-changed was not emitted".into(),
        ))
    }

    /// Fetch historical logs as bounded JSONL.
    pub async fn logs_fetch(
        &self,
        request: &LogFetchRequest,
    ) -> Result<LogFetchResponse, DozzleError> {
        let limited = self.fetch_logs_text(request, None).await?;
        let limits = request.limits.clone().capped();
        let mut events = Vec::new();
        let mut truncated = limited.truncated;
        let mut limit_kind = limited.limit_kind;
        for line in limited.text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if events.len() >= limits.max_lines {
                truncated = true;
                limit_kind = Some("max_lines".to_string());
                break;
            }
            let event = serde_json::from_str::<Value>(line)
                .map_err(|e| DozzleError::InvalidResponse(format!("invalid JSONL log row: {e}")))?;
            events.push(event);
        }
        let lines_read = events.len();
        Ok(LogFetchResponse {
            events,
            meta: LimitMetadata {
                truncated,
                limit_kind,
                bytes_read: limited.bytes_read,
                events_read: None,
                lines_read: Some(lines_read),
                duration_ms: limited.duration_ms,
            },
        })
    }

    /// Fetch historical logs as bounded plain text.
    pub async fn logs_fetch_plain(
        &self,
        request: &LogFetchRequest,
    ) -> Result<PlainLogFetchResponse, DozzleError> {
        let limited = self.fetch_logs_text(request, Some("text/plain")).await?;
        let lines_read = limited.text.lines().count();
        Ok(PlainLogFetchResponse {
            text: limited.text,
            meta: LimitMetadata {
                truncated: limited.truncated,
                limit_kind: limited.limit_kind,
                bytes_read: limited.bytes_read,
                events_read: None,
                lines_read: Some(lines_read),
                duration_ms: limited.duration_ms,
            },
        })
    }

    async fn fetch_logs_text(
        &self,
        request: &LogFetchRequest,
        accept: Option<&str>,
    ) -> Result<LimitedText, DozzleError> {
        if !request.stdout && !request.stderr {
            return Err(DozzleError::InvalidResponse(
                "stdout or stderr must be selected".into(),
            ));
        }
        let host = HttpClient::encode_path_segment(&request.host);
        let container_id = HttpClient::encode_path_segment(&request.container_id);
        let path = format!("/api/hosts/{host}/containers/{container_id}/logs");
        let mut query = Vec::new();
        if request.stdout {
            query.push(("stdout".to_string(), "1".to_string()));
        }
        if request.stderr {
            query.push(("stderr".to_string(), "1".to_string()));
        }
        query.push(("everything".to_string(), "1".to_string()));
        let url = self.url_with_query(&path, &query)?;
        self.read_limited_text(url, request.limits.clone(), accept)
            .await
    }

    fn url_with_query(&self, path: &str, query: &[(String, String)]) -> Result<Url, DozzleError> {
        if path.starts_with("http://") || path.starts_with("https://") {
            return Err(ApiError::Internal(format!("absolute URL not permitted: {path}")).into());
        }
        let raw = if path.starts_with('/') {
            format!("{}{path}", self.base_url.trim_end_matches('/'))
        } else {
            format!("{}/{path}", self.base_url.trim_end_matches('/'))
        };
        let mut url =
            Url::parse(&raw).map_err(|e| ApiError::Internal(format!("invalid url: {e}")))?;
        if !query.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in query {
                pairs.append_pair(key, value);
            }
        }
        Ok(url)
    }

    fn apply_auth(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.auth {
            Auth::None => request,
            Auth::ApiKey { header, key } => request.header(header, key),
            Auth::Token { token } => request.header("Authorization", format!("Token {token}")),
            Auth::Bearer { token } => request.bearer_auth(token),
            Auth::Basic { username, password } => request.basic_auth(username, Some(password)),
            Auth::Session { cookie } => request.header("Cookie", cookie),
        }
    }

    async fn read_limited_text(
        &self,
        url: Url,
        limits: ReadLimits,
        accept: Option<&str>,
    ) -> Result<LimitedText, DozzleError> {
        let started = Instant::now();
        let limits = limits.capped();
        let mut request = self.inner.get(url);
        if let Some(accept) = accept {
            request = request.header("Accept", accept);
        }
        let response = tokio::time::timeout(
            Duration::from_millis(limits.timeout_ms),
            self.apply_auth(request).send(),
        )
        .await
        .map_err(|_| DozzleError::StreamTimeout("request timed out".into()))?
        .map_err(|e| ApiError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::error_for_status(response.status().as_u16()).into());
        }

        let mut stream = response.bytes_stream();
        let mut bytes = Vec::new();
        let mut truncated = false;
        let mut limit_kind = None;
        let read = async {
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|e| ApiError::Network(e.to_string()))?;
                if bytes.len().saturating_add(chunk.len()) > limits.max_bytes {
                    let remaining = limits.max_bytes.saturating_sub(bytes.len());
                    bytes.extend_from_slice(&chunk[..remaining]);
                    truncated = true;
                    limit_kind = Some("max_bytes".to_string());
                    break;
                }
                bytes.extend_from_slice(&chunk);
            }
            Ok::<(), DozzleError>(())
        };
        tokio::time::timeout(Duration::from_millis(limits.timeout_ms), read)
            .await
            .map_err(|_| DozzleError::StreamTimeout("read timed out".into()))??;

        let bytes_read = bytes.len();
        let text =
            String::from_utf8(bytes).map_err(|e| DozzleError::InvalidResponse(e.to_string()))?;
        Ok(LimitedText {
            text,
            bytes_read,
            truncated,
            limit_kind,
            duration_ms: started.elapsed().as_millis(),
        })
    }

    async fn read_sse_events(
        &self,
        url: Url,
        limits: &ReadLimits,
    ) -> Result<Vec<ParsedSseEvent>, DozzleError> {
        let limits = limits.clone().capped();
        let response = self
            .apply_auth(self.inner.get(url).header("Accept", "text/event-stream"))
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::error_for_status(response.status().as_u16()).into());
        }

        let read = async move {
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut bytes_read = 0usize;
            let mut current_event: Option<String> = None;
            let mut current_data = String::new();
            let mut events = Vec::new();

            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|e| ApiError::Network(e.to_string()))?;
                bytes_read = bytes_read.saturating_add(chunk.len());
                if bytes_read > limits.max_bytes {
                    break;
                }
                buffer.push_str(
                    std::str::from_utf8(&chunk)
                        .map_err(|e| DozzleError::InvalidResponse(e.to_string()))?,
                );

                while let Some(pos) = buffer.find('\n') {
                    let mut line = buffer[..pos].trim_end_matches('\r').to_string();
                    buffer.drain(..=pos);
                    if line.is_empty() {
                        if let Some(event) = current_event.take() {
                            let data =
                                serde_json::from_str::<Value>(&current_data).map_err(|e| {
                                    DozzleError::InvalidResponse(format!(
                                        "invalid SSE JSON data: {e}"
                                    ))
                                })?;
                            events.push(ParsedSseEvent {
                                event,
                                data,
                                bytes_read,
                            });
                            current_data.clear();
                            if events.len() >= limits.max_events {
                                return Ok(events);
                            }
                        }
                        continue;
                    }
                    if let Some(rest) = line.strip_prefix("event:") {
                        current_event = Some(rest.trim().to_string());
                    } else if let Some(rest) = line.strip_prefix("data:") {
                        if !current_data.is_empty() {
                            current_data.push('\n');
                        }
                        current_data.push_str(rest.trim());
                    }
                    line.clear();
                }
            }
            Ok(events)
        };

        tokio::time::timeout(Duration::from_millis(limits.timeout_ms), read)
            .await
            .map_err(|_| DozzleError::StreamTimeout("SSE read timed out".into()))?
    }

    fn error_for_status(status: u16) -> ApiError {
        match status {
            401 | 403 => ApiError::Auth,
            404 => ApiError::NotFound,
            429 => ApiError::RateLimited { retry_after: None },
            _ => ApiError::Server {
                status,
                body: "<redacted upstream error body>".to_string(),
            },
        }
    }
}

#[derive(Debug)]
struct ParsedSseEvent {
    event: String,
    data: Value,
    bytes_read: usize,
}
