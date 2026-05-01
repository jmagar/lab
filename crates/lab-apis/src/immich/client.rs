//! Immich read-only API methods.

use crate::core::{Auth, HttpClient};

use super::error::ImmichError;
use super::types::{AssetMetadata, AssetSearchRequest, AssetSearchResponse, ServerInfo, UserMe};

const MAX_ASSET_LIMIT: u32 = 50;

#[derive(Clone)]
pub struct ImmichClient {
    http: HttpClient,
}

impl ImmichClient {
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, ImmichError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    pub async fn health(&self) -> Result<(), ImmichError> {
        self.http.get_void("/api/server/ping").await?;
        Ok(())
    }

    pub async fn server_info(&self) -> Result<ServerInfo, ImmichError> {
        Ok(ServerInfo {
            value: self.http.get_json("/api/server/about").await?,
        })
    }

    pub async fn server_version(&self) -> Result<ServerInfo, ImmichError> {
        Ok(ServerInfo {
            value: self.http.get_json("/api/server/version").await?,
        })
    }

    pub async fn user_me(&self) -> Result<UserMe, ImmichError> {
        Ok(UserMe {
            value: self.http.get_json("/api/users/me").await?,
        })
    }

    pub async fn asset_search(
        &self,
        request: AssetSearchRequest,
    ) -> Result<AssetSearchResponse, ImmichError> {
        if request.limit == 0 || request.limit > MAX_ASSET_LIMIT {
            return Err(ImmichError::InvalidParam(format!(
                "limit must be between 1 and {MAX_ASSET_LIMIT}"
            )));
        }
        let mut body = serde_json::Map::new();
        body.insert("size".to_string(), serde_json::json!(request.limit));
        if let Some(page) = request.page {
            body.insert("page".to_string(), serde_json::json!(page));
        }
        if let Some(query) = request.query {
            body.insert("query".to_string(), serde_json::json!(query));
        }
        let raw: serde_json::Value = self
            .http
            .post_json("/api/search/metadata", &serde_json::Value::Object(body))
            .await?;
        let assets = raw
            .get("assets")
            .and_then(|v| v.get("items"))
            .or_else(|| raw.get("items"))
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        let mut items: Vec<_> = assets.into_iter().map(redact_asset).collect();
        let truncated = items.len() > request.limit as usize;
        items.truncate(request.limit as usize);
        Ok(AssetSearchResponse {
            items,
            page: raw
                .get("assets")
                .and_then(|v| v.get("page"))
                .or_else(|| raw.get("page"))
                .and_then(serde_json::Value::as_u64)
                .and_then(|n| u32::try_from(n).ok()),
            next_page: raw
                .get("assets")
                .and_then(|v| v.get("nextPage"))
                .or_else(|| raw.get("nextPage"))
                .and_then(serde_json::Value::as_u64)
                .and_then(|n| u32::try_from(n).ok()),
            truncated,
        })
    }

    pub async fn asset_get(&self, id: &str) -> Result<AssetMetadata, ImmichError> {
        let id = HttpClient::encode_path_segment(id);
        let value: serde_json::Value = self.http.get_json(&format!("/api/assets/{id}")).await?;
        Ok(AssetMetadata {
            value: redact_asset(value),
        })
    }
}

fn redact_asset(mut value: serde_json::Value) -> serde_json::Value {
    let Some(object) = value.as_object_mut() else {
        return value;
    };
    for key in [
        "exifInfo",
        "faces",
        "people",
        "owner",
        "ownerId",
        "originalPath",
        "originalFileName",
        "thumbhash",
        "livePhotoVideoId",
        "localDateTime",
        "checksum",
    ] {
        object.remove(key);
    }
    value
}
