//! AdGuard Home read-first API methods.

use crate::core::{Auth, HttpClient};

use super::error::AdguardError;
use super::types::{AdguardResponse, QueryLogResponse};

const MAX_QUERYLOG_LIMIT: u32 = 200;

#[derive(Clone)]
pub struct AdguardClient {
    http: HttpClient,
    base_url: String,
    inner: reqwest::Client,
    username: Option<String>,
    password: Option<String>,
}

impl AdguardClient {
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, AdguardError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
            base_url: base_url.to_string(),
            inner: reqwest::Client::builder()
                .user_agent(concat!("lab-apis/", env!("CARGO_PKG_VERSION")))
                .build()
                .map_err(|e| crate::core::error::ApiError::Internal(e.to_string()))?,
            username: None,
            password: None,
        })
    }

    pub fn with_login(
        base_url: &str,
        username: String,
        password: String,
    ) -> Result<Self, AdguardError> {
        let mut client = Self::new(base_url, Auth::None)?;
        client.username = Some(username);
        client.password = Some(password);
        Ok(client)
    }

    pub async fn status(&self) -> Result<AdguardResponse, AdguardError> {
        Ok(AdguardResponse {
            value: self.authed_get_json("/control/status", &[]).await?,
        })
    }

    pub async fn version(&self) -> Result<AdguardResponse, AdguardError> {
        Ok(AdguardResponse {
            value: serde_json::json!({ "version": self.http.get_text("/control/version").await? }),
        })
    }

    pub async fn stats(&self) -> Result<AdguardResponse, AdguardError> {
        Ok(AdguardResponse {
            value: self.authed_get_json("/control/stats", &[]).await?,
        })
    }

    pub async fn querylog(
        &self,
        limit: u32,
        search: Option<&str>,
        older_than: Option<&str>,
    ) -> Result<QueryLogResponse, AdguardError> {
        if limit == 0 || limit > MAX_QUERYLOG_LIMIT {
            return Err(AdguardError::InvalidParam(format!(
                "limit must be between 1 and {MAX_QUERYLOG_LIMIT}"
            )));
        }
        let mut query = vec![("limit".to_string(), limit.to_string())];
        if let Some(search) = search {
            query.push(("search".to_string(), search.to_string()));
        }
        if let Some(older_than) = older_than {
            query.push(("older_than".to_string(), older_than.to_string()));
        }
        Ok(QueryLogResponse {
            value: redact_querylog(self.authed_get_json("/control/querylog", &query).await?),
            limit,
        })
    }

    pub async fn filtering_status(&self) -> Result<AdguardResponse, AdguardError> {
        Ok(AdguardResponse {
            value: self
                .authed_get_json("/control/filtering/status", &[])
                .await?,
        })
    }

    pub async fn filtering_check_host(&self, host: &str) -> Result<AdguardResponse, AdguardError> {
        Ok(AdguardResponse {
            value: self
                .authed_get_json(
                    "/control/filtering/check_host",
                    &[("name".to_string(), host.to_string())],
                )
                .await?,
        })
    }

    async fn authed_get_json(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<serde_json::Value, AdguardError> {
        if self.username.is_none() {
            return Ok(self.http.get_json_query(path, query).await?);
        }
        let cookie = self.login_cookie().await?;
        let client = HttpClient::new(&self.base_url, Auth::Session { cookie })?;
        Ok(client.get_json_query(path, query).await?)
    }

    async fn login_cookie(&self) -> Result<String, AdguardError> {
        let Some(username) = &self.username else {
            return Err(AdguardError::InvalidParam("username not configured".into()));
        };
        let Some(password) = &self.password else {
            return Err(AdguardError::InvalidParam("password not configured".into()));
        };
        let url = format!("{}/control/login", self.base_url.trim_end_matches('/'));
        let response = self
            .inner
            .post(url)
            .json(&serde_json::json!({ "name": username, "password": password }))
            .send()
            .await
            .map_err(|e| crate::core::error::ApiError::Network(e.to_string()))?;
        if !response.status().is_success() {
            return Err(crate::core::error::ApiError::Auth.into());
        }
        response
            .headers()
            .get(reqwest::header::SET_COOKIE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(';').next())
            .map(str::to_string)
            .ok_or_else(|| AdguardError::InvalidParam("login did not return session cookie".into()))
    }
}

fn redact_querylog(mut value: serde_json::Value) -> serde_json::Value {
    redact_recursive(&mut value);
    value
}

fn redact_recursive(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for key in ["client", "client_ip", "client_proto", "upstream"] {
                map.remove(key);
            }
            for value in map.values_mut() {
                redact_recursive(value);
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                redact_recursive(value);
            }
        }
        _ => {}
    }
}
