//! FreshRSS Google Reader-compatible read methods.

use crate::core::error::ApiError;
use crate::core::{Auth, HttpClient};

use super::error::FreshrssError;
use super::types::FreshRssResponse;

const MAX_ITEMS: u32 = 100;

#[derive(Clone)]
pub struct FreshrssClient {
    http: HttpClient,
    base_url: String,
    inner: reqwest::Client,
    username: String,
    password: String,
}

impl FreshrssClient {
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, FreshrssError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
            base_url: base_url.to_string(),
            inner: reqwest::Client::builder()
                .user_agent(concat!("lab-apis/", env!("CARGO_PKG_VERSION")))
                .build()
                .map_err(|e| ApiError::Internal(format!("reqwest::Client::build: {e}")))?,
            username: String::new(),
            password: String::new(),
        })
    }

    pub fn with_credentials(
        base_url: &str,
        username: String,
        password: String,
    ) -> Result<Self, FreshrssError> {
        Ok(Self {
            http: HttpClient::new(base_url, Auth::None)?,
            base_url: base_url.to_string(),
            inner: reqwest::Client::builder()
                .user_agent(concat!("lab-apis/", env!("CARGO_PKG_VERSION")))
                .build()
                .map_err(|e| ApiError::Internal(format!("reqwest::Client::build: {e}")))?,
            username,
            password,
        })
    }

    pub async fn subscriptions(&self) -> Result<FreshRssResponse, FreshrssError> {
        self.get_json("/reader/api/0/subscription/list", &[]).await
    }

    pub async fn health(&self) -> Result<(), FreshrssError> {
        self.subscriptions().await?;
        Ok(())
    }

    pub async fn tags(&self) -> Result<FreshRssResponse, FreshrssError> {
        self.get_json("/reader/api/0/tag/list", &[]).await
    }

    pub async fn unread_counts(&self) -> Result<FreshRssResponse, FreshrssError> {
        self.get_json("/reader/api/0/unread-count", &[]).await
    }

    pub async fn stream_items(
        &self,
        n: u32,
        continuation: Option<&str>,
    ) -> Result<FreshRssResponse, FreshrssError> {
        if n == 0 || n > MAX_ITEMS {
            return Err(FreshrssError::InvalidParam(format!(
                "n must be between 1 and {MAX_ITEMS}"
            )));
        }
        let mut query = vec![("n".to_string(), n.to_string())];
        if let Some(c) = continuation {
            query.push(("c".to_string(), c.to_string()));
        }
        self.get_json("/reader/api/0/stream/contents/reading-list", &query)
            .await
    }

    async fn get_json(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<FreshRssResponse, FreshrssError> {
        let token = self.login().await?;
        let auth = Auth::Token { token };
        let client = HttpClient::new(self.http.base_url(), auth)?;
        Ok(FreshRssResponse {
            value: client.get_json_query(path, query).await?,
        })
    }

    async fn login(&self) -> Result<String, FreshrssError> {
        let url = format!(
            "{}/accounts/ClientLogin",
            self.base_url.trim_end_matches('/')
        );
        let fields = [
            ("Email", self.username.as_str()),
            ("Passwd", self.password.as_str()),
            ("service", "reader"),
            ("accountType", "HOSTED_OR_GOOGLE"),
        ];
        let response = self
            .inner
            .post(url)
            .form(&fields)
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        if !response.status().is_success() {
            return Err(ApiError::Auth.into());
        }
        let text = response
            .text()
            .await
            .map_err(|e| ApiError::Decode(e.to_string()))?;
        parse_client_login_token(&text).ok_or(FreshrssError::MissingAuthToken)
    }
}

fn parse_client_login_token(text: &str) -> Option<String> {
    text.lines()
        .find_map(|line| line.strip_prefix("Auth="))
        .map(ToString::to_string)
}
