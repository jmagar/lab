use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use reqwest::header::{CONTENT_TYPE, COOKIE, HeaderMap, HeaderValue};
use serde_json::{Value, json};
use tokio::sync::RwLock;
use tracing::{Level, event};
use url::form_urlencoded;

use crate::core::error::ApiError;

use super::error::NotebookLmError;
use super::types::{
    ADD_SOURCE, CREATE_NOTEBOOK, DELETE_NOTEBOOK, GET_NOTEBOOK, LIST_NOTEBOOKS, Notebook,
    NotebookLmAuth, Source, notebook_from_api, source_from_api, sources_from_notebook_response,
};

const BASE_URL: &str = "https://notebooklm.google.com";
const BATCHEXECUTE_PATH: &str = "/_/LabsTailwindUi/data/batchexecute";

#[derive(Debug, Clone)]
struct Tokens {
    csrf: String,
    session_id: String,
}

#[derive(Debug, Clone)]
pub struct NotebookLmClient {
    http: reqwest::Client,
    base_url: String,
    tokens: Arc<RwLock<Option<Tokens>>>,
}

impl NotebookLmClient {
    /// Build a NotebookLM client from Playwright `storage_state.json` content.
    ///
    /// # Errors
    /// Returns an error when JSON is invalid, required Google cookies are
    /// missing, or the HTTP client cannot be initialized.
    pub fn from_storage_json(storage_json: &str) -> Result<Self, NotebookLmError> {
        let auth: NotebookLmAuth = serde_json::from_str(storage_json)
            .map_err(|e| ApiError::Decode(format!("invalid NotebookLM storage JSON: {e}")))?;
        Self::from_auth(auth)
    }

    /// Build a NotebookLM client from parsed Playwright storage state.
    ///
    /// # Errors
    /// Returns an error when no Google `SID` cookie is available.
    pub fn from_auth(auth: NotebookLmAuth) -> Result<Self, NotebookLmError> {
        Self::from_auth_with_base_url(auth, BASE_URL)
    }

    fn from_auth_with_base_url(
        auth: NotebookLmAuth,
        base_url: &str,
    ) -> Result<Self, NotebookLmError> {
        let cookies = auth_cookies(&auth)?;
        let http = reqwest::Client::builder()
            .user_agent(concat!("lab-apis/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(30))
            .default_headers(default_headers(&cookies)?)
            .build()
            .map_err(|e| ApiError::Internal(format!("reqwest::Client::build: {e}")))?;
        Ok(Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
            tokens: Arc::new(RwLock::new(None)),
        })
    }

    /// List all notebooks.
    ///
    /// # Errors
    /// Returns `NotebookLmError` on auth, transport, or decode failure.
    pub async fn list_notebooks(&self) -> Result<Vec<Notebook>, NotebookLmError> {
        let result = self
            .rpc(LIST_NOTEBOOKS, json!([null, 1, null, [2]]), "/", false)
            .await?;
        let notebooks = result
            .as_array()
            .and_then(|outer| outer.first())
            .and_then(Value::as_array)
            .map(|items| items.iter().map(notebook_from_api).collect())
            .unwrap_or_default();
        Ok(notebooks)
    }

    /// Create a notebook.
    ///
    /// # Errors
    /// Returns `NotebookLmError` on auth, transport, or decode failure.
    pub async fn create_notebook(&self, title: &str) -> Result<Notebook, NotebookLmError> {
        let result = self
            .rpc(
                CREATE_NOTEBOOK,
                json!([title, null, null, [2], [1]]),
                "/",
                false,
            )
            .await?;
        Ok(notebook_from_api(&result))
    }

    /// Get notebook details.
    ///
    /// # Errors
    /// Returns `NotebookLmError` on auth, transport, or decode failure.
    pub async fn get_notebook(&self, notebook_id: &str) -> Result<Notebook, NotebookLmError> {
        let result = self
            .rpc(
                GET_NOTEBOOK,
                json!([notebook_id, null, [2], null, 0]),
                &format!("/notebook/{notebook_id}"),
                false,
            )
            .await?;
        let data = result
            .as_array()
            .and_then(|items| items.first())
            .ok_or_else(|| ApiError::Decode("NotebookLM get returned no notebook".to_string()))?;
        Ok(notebook_from_api(data))
    }

    /// Delete a notebook.
    ///
    /// # Errors
    /// Returns `NotebookLmError` on auth, transport, or decode failure.
    pub async fn delete_notebook(&self, notebook_id: &str) -> Result<bool, NotebookLmError> {
        self.rpc(DELETE_NOTEBOOK, json!([[notebook_id], [2]]), "/", true)
            .await?;
        Ok(true)
    }

    /// List sources for a notebook.
    ///
    /// # Errors
    /// Returns `NotebookLmError` on auth, transport, or decode failure.
    pub async fn list_sources(&self, notebook_id: &str) -> Result<Vec<Source>, NotebookLmError> {
        let result = self
            .rpc(
                GET_NOTEBOOK,
                json!([notebook_id, null, [2], null, 0]),
                &format!("/notebook/{notebook_id}"),
                false,
            )
            .await?;
        Ok(sources_from_notebook_response(&result))
    }

    /// Add a regular URL source.
    ///
    /// # Errors
    /// Returns `NotebookLmError` on auth, transport, or decode failure.
    pub async fn add_url_source(
        &self,
        notebook_id: &str,
        url: &str,
    ) -> Result<Source, NotebookLmError> {
        let result = self
            .rpc(
                ADD_SOURCE,
                json!([
                    [[null, null, [url], null, null, null, null, null]],
                    notebook_id,
                    [2],
                    null,
                    null
                ]),
                &format!("/notebook/{notebook_id}"),
                false,
            )
            .await?;
        source_from_api(&result).ok_or_else(|| {
            ApiError::Decode("NotebookLM add source returned no source".to_string()).into()
        })
    }

    async fn rpc(
        &self,
        method: &str,
        params: Value,
        source_path: &str,
        allow_null: bool,
    ) -> Result<Value, NotebookLmError> {
        let tokens = self.tokens().await?;
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("rpcids", method)
            .append_pair("source-path", source_path)
            .append_pair("f.sid", &tokens.session_id)
            .append_pair("rt", "c")
            .finish();
        let rpc_request = json!([[[method, compact_json(&params)?, null, "generic"]]]);
        let form = form_urlencoded::Serializer::new(String::new())
            .append_pair("f.req", &compact_json(&rpc_request)?)
            .append_pair("at", &tokens.csrf)
            .finish();
        let url = format!("{}{BATCHEXECUTE_PATH}?{query}", self.base_url);

        let request_log = RequestLog::new("POST", BATCHEXECUTE_PATH, &self.base_url);
        request_log.start();
        let response = self
            .http
            .post(url)
            .body(format!("{form}&"))
            .send()
            .await
            .map_err(|e| {
                let err = ApiError::Network(e.to_string());
                request_log.error(&err);
                err
            })?;
        let status = response.status();
        let text = response.text().await.map_err(|e| {
            let err = ApiError::Network(e.to_string());
            request_log.error(&err);
            err
        })?;
        if !status.is_success() {
            let err = status_error(status.as_u16(), text);
            request_log.error(&err);
            return Err(err.into());
        }
        let decoded = decode_rpc_response(&text, method, allow_null);
        match &decoded {
            Ok(_) => request_log.finish(status.as_u16()),
            Err(err) => request_log.error(err),
        }
        decoded.map_err(Into::into)
    }

    async fn tokens(&self) -> Result<Tokens, NotebookLmError> {
        if let Some(tokens) = self.tokens.read().await.clone() {
            return Ok(tokens);
        }

        let request_log = RequestLog::new("GET", "/", &self.base_url);
        request_log.start();
        let response = self.http.get(&self.base_url).send().await.map_err(|e| {
            let err = ApiError::Network(e.to_string());
            request_log.error(&err);
            err
        })?;
        let final_url = response.url().to_string();
        let status = response.status();
        let html = response.text().await.map_err(|e| {
            let err = ApiError::Network(e.to_string());
            request_log.error(&err);
            err
        })?;
        if !status.is_success() {
            let err = status_error(status.as_u16(), html);
            request_log.error(&err);
            return Err(err.into());
        }
        if final_url.contains("accounts.google.com") {
            let err = ApiError::Auth;
            request_log.error(&err);
            return Err(err.into());
        }

        let tokens = match (
            extract_global_value(&html, "SNlM0e"),
            extract_global_value(&html, "FdrFJe"),
        ) {
            (Ok(csrf), Ok(session_id)) => Tokens { csrf, session_id },
            (Err(err), _) | (_, Err(err)) => {
                request_log.error(&err);
                return Err(err.into());
            }
        };
        request_log.finish(status.as_u16());
        *self.tokens.write().await = Some(tokens.clone());
        Ok(tokens)
    }
}

struct RequestLog {
    method: &'static str,
    path: &'static str,
    host: String,
    start: Instant,
}

impl RequestLog {
    fn new(method: &'static str, path: &'static str, base_url: &str) -> Self {
        let host = url::Url::parse(base_url)
            .ok()
            .and_then(|url| url.host_str().map(ToString::to_string))
            .unwrap_or_else(|| "notebooklm.google.com".to_string());
        Self {
            method,
            path,
            host,
            start: Instant::now(),
        }
    }

    fn start(&self) {
        event!(
            Level::INFO,
            method = self.method,
            path = self.path,
            host = self.host.as_str(),
            "request.start"
        );
    }

    fn finish(&self, status: u16) {
        event!(
            Level::INFO,
            method = self.method,
            path = self.path,
            host = self.host.as_str(),
            status,
            elapsed_ms = self.elapsed_ms(),
            "request.finish"
        );
    }

    fn error(&self, error: &ApiError) {
        event!(
            Level::WARN,
            method = self.method,
            path = self.path,
            host = self.host.as_str(),
            kind = error.kind(),
            elapsed_ms = self.elapsed_ms(),
            "request.error"
        );
    }

    fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }
}

fn default_headers(cookies: &str) -> Result<HeaderMap, ApiError> {
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/x-www-form-urlencoded;charset=UTF-8"),
    );
    headers.insert(
        COOKIE,
        HeaderValue::from_str(cookies).map_err(|e| ApiError::Validation {
            field: "cookies".to_string(),
            message: e.to_string(),
        })?,
    );
    Ok(headers)
}

fn auth_cookies(auth: &NotebookLmAuth) -> Result<String, ApiError> {
    let mut cookies = BTreeMap::new();
    for cookie in &auth.cookies {
        if !cookie.name.is_empty()
            && !cookie.value.is_empty()
            && allowed_domain(cookie.domain.as_deref())
        {
            cookies
                .entry(cookie.name.as_str())
                .or_insert(cookie.value.as_str());
        }
    }
    if !cookies.contains_key("SID") {
        return Err(ApiError::Auth);
    }
    Ok(cookies
        .into_iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("; "))
}

fn allowed_domain(domain: Option<&str>) -> bool {
    let Some(domain) = domain else {
        return false;
    };
    domain == "google.com"
        || domain == ".google.com"
        || domain == ".notebooklm.google.com"
        || domain == "notebooklm.google.com"
}

fn extract_global_value(html: &str, key: &str) -> Result<String, ApiError> {
    let needle = format!("\"{key}\"");
    let Some(start) = html.find(&needle) else {
        return Err(ApiError::Auth);
    };
    let after_key = &html[start + needle.len()..];
    let Some(colon) = after_key.find(':') else {
        return Err(ApiError::Decode(format!(
            "NotebookLM token {key} missing colon"
        )));
    };
    let after_colon = after_key[colon + 1..].trim_start();
    let Some(stripped) = after_colon.strip_prefix('"') else {
        return Err(ApiError::Decode(format!(
            "NotebookLM token {key} is not a string"
        )));
    };
    let Some(end) = stripped.find('"') else {
        return Err(ApiError::Decode(format!(
            "NotebookLM token {key} is unterminated"
        )));
    };
    Ok(stripped[..end].to_string())
}

fn compact_json(value: &Value) -> Result<String, ApiError> {
    serde_json::to_string(value).map_err(|e| ApiError::Decode(e.to_string()))
}

fn decode_rpc_response(text: &str, method: &str, allow_null: bool) -> Result<Value, ApiError> {
    let clean = text
        .strip_prefix(")]}'\n")
        .or_else(|| text.strip_prefix(")]}'\r\n"))
        .unwrap_or(text);
    for line in clean.lines() {
        let line = line.trim();
        if line.is_empty() || line.parse::<usize>().is_ok() {
            continue;
        }
        let Ok(chunk) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if let Some(result) = extract_result(&chunk, method, allow_null)? {
            if result.is_null() && !allow_null {
                return Err(ApiError::Decode(format!(
                    "NotebookLM RPC {method} returned null result"
                )));
            }
            return Ok(result);
        }
    }
    Err(ApiError::Decode(format!(
        "NotebookLM RPC response did not contain method {method}"
    )))
}

fn extract_result(
    value: &Value,
    method: &str,
    allow_null: bool,
) -> Result<Option<Value>, ApiError> {
    let Some(arr) = value.as_array() else {
        return Ok(None);
    };
    let items: Vec<&Value> = if arr.first().and_then(Value::as_array).is_some() {
        arr.iter().collect()
    } else {
        vec![value]
    };
    for item in items {
        let Some(item) = item.as_array() else {
            continue;
        };
        if item.first().and_then(Value::as_str) == Some("er")
            && item.get(1).and_then(Value::as_str) == Some(method)
        {
            return Err(
                classify_rpc_error(item, 2).unwrap_or_else(|| ApiError::Server {
                    status: 200,
                    body: sanitize_upstream_body(
                        item.get(2)
                            .map(ToString::to_string)
                            .unwrap_or_else(|| "NotebookLM RPC error".to_string()),
                    ),
                }),
            );
        }
        if item.first().and_then(Value::as_str) == Some("wrb.fr")
            && item.get(1).and_then(Value::as_str) == Some(method)
        {
            let Some(result) = item.get(2) else {
                return Ok(Some(Value::Null));
            };
            if result.is_null()
                && !allow_null
                && let Some(err) = classify_rpc_error(item, 5)
            {
                return Err(err);
            }
            if let Some(encoded) = result.as_str() {
                let parsed = serde_json::from_str::<Value>(encoded)
                    .map_err(|e| ApiError::Decode(e.to_string()))?;
                return Ok(Some(parsed));
            }
            return Ok(Some(result.clone()));
        }
    }
    Ok(None)
}

fn status_error(status: u16, body: String) -> ApiError {
    match status {
        401 | 403 => ApiError::Auth,
        404 => ApiError::NotFound,
        429 => ApiError::RateLimited { retry_after: None },
        _ => ApiError::Server {
            status,
            body: sanitize_upstream_body(body),
        },
    }
}

fn classify_rpc_error(item: &[Value], index: usize) -> Option<ApiError> {
    let text = item.get(index)?.to_string();
    let lower = text.to_ascii_lowercase();
    if lower.contains("rate") || lower.contains("quota") || lower.contains("resource_exhausted") {
        return Some(ApiError::RateLimited { retry_after: None });
    }
    if lower.contains("not_found") || lower.contains("not found") {
        return Some(ApiError::NotFound);
    }
    if lower.contains("permission")
        || lower.contains("unauth")
        || lower.contains("login")
        || lower.contains("forbidden")
    {
        return Some(ApiError::Auth);
    }
    Some(ApiError::Validation {
        field: "notebooklm".to_string(),
        message: sanitize_upstream_body(text),
    })
}

fn sanitize_upstream_body(body: String) -> String {
    let clean = body
        .chars()
        .filter(|ch| !ch.is_control() || *ch == '\n' || *ch == '\t')
        .collect::<String>();
    let trimmed = clean.trim();
    const MAX_BODY_CHARS: usize = 240;
    if trimmed.chars().count() <= MAX_BODY_CHARS {
        return trimmed.to_string();
    }
    let mut truncated = trimmed.chars().take(MAX_BODY_CHARS).collect::<String>();
    truncated.push_str("...");
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_json_requires_sid_cookie() {
        let err = NotebookLmClient::from_storage_json(r#"{"cookies":[]}"#).unwrap_err();
        assert!(matches!(err, NotebookLmError::Api(ApiError::Auth)));
    }

    #[test]
    fn storage_json_ignores_cookie_without_domain() {
        let err = NotebookLmClient::from_storage_json(
            r#"{"cookies":[{"name":"SID","value":"sid-value"}]}"#,
        )
        .unwrap_err();
        assert!(matches!(err, NotebookLmError::Api(ApiError::Auth)));
    }

    #[test]
    fn storage_json_accepts_google_parent_domain_cookie() {
        NotebookLmClient::from_storage_json(
            r#"{"cookies":[{"name":"SID","value":"sid-value","domain":".google.com"}]}"#,
        )
        .unwrap();
    }

    #[test]
    fn extracts_tokens_from_homepage_html() {
        let html = r#"window.WIZ_global_data = {"SNlM0e":"csrf-token","FdrFJe":"session-id"};"#;
        assert_eq!(extract_global_value(html, "SNlM0e").unwrap(), "csrf-token");
        assert_eq!(extract_global_value(html, "FdrFJe").unwrap(), "session-id");
    }

    #[test]
    fn decodes_chunked_batchexecute_response() {
        let response = ")]}'\n42\n[[\"wrb.fr\",\"wXbhsf\",\"[[[\\\"Title\\\",null,\\\"nb-1\\\"]]]\",null,null,null]]\n";
        let decoded = decode_rpc_response(response, LIST_NOTEBOOKS, false).unwrap();
        assert_eq!(decoded[0][0][0], "Title");
        assert_eq!(decoded[0][0][2], "nb-1");
    }
}
