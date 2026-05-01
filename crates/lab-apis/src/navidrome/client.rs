//! Navidrome Subsonic-compatible read methods.

use crate::core::{Auth, HttpClient};

use super::error::NavidromeError;
use super::types::{QueryResponse, SubsonicResponse};

const MAX_SIZE: u32 = 100;

#[derive(Clone)]
pub struct NavidromeClient {
    http: HttpClient,
    username: String,
    token: String,
    salt: String,
    client_name: String,
}

impl NavidromeClient {
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, NavidromeError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
            username: String::new(),
            token: String::new(),
            salt: String::new(),
            client_name: "lab".to_string(),
        })
    }

    pub fn with_subsonic_auth(
        base_url: &str,
        username: String,
        token: String,
        salt: String,
    ) -> Result<Self, NavidromeError> {
        Ok(Self {
            http: HttpClient::new(base_url, Auth::None)?,
            username,
            token,
            salt,
            client_name: "lab".to_string(),
        })
    }

    pub async fn ping(&self) -> Result<QueryResponse, NavidromeError> {
        self.get("ping", &[]).await
    }

    pub async fn health(&self) -> Result<(), NavidromeError> {
        self.ping().await?;
        Ok(())
    }

    pub async fn artists(&self) -> Result<QueryResponse, NavidromeError> {
        self.get("getArtists", &[]).await
    }

    pub async fn album_list(
        &self,
        list_type: &str,
        size: u32,
        offset: u32,
    ) -> Result<QueryResponse, NavidromeError> {
        let size = clamp_size(size)?;
        self.get(
            "getAlbumList2",
            &[
                ("type".to_string(), list_type.to_string()),
                ("size".to_string(), size.to_string()),
                ("offset".to_string(), offset.to_string()),
            ],
        )
        .await
    }

    pub async fn album_get(&self, id: &str) -> Result<QueryResponse, NavidromeError> {
        self.get("getAlbum", &[("id".to_string(), id.to_string())])
            .await
    }

    pub async fn search(
        &self,
        query: &str,
        size: u32,
        offset: u32,
    ) -> Result<QueryResponse, NavidromeError> {
        let size = clamp_size(size)?;
        self.get(
            "search3",
            &[
                ("query".to_string(), query.to_string()),
                ("artistCount".to_string(), size.to_string()),
                ("artistOffset".to_string(), offset.to_string()),
                ("albumCount".to_string(), size.to_string()),
                ("albumOffset".to_string(), offset.to_string()),
                ("songCount".to_string(), size.to_string()),
                ("songOffset".to_string(), offset.to_string()),
            ],
        )
        .await
    }

    pub async fn playlists(&self) -> Result<QueryResponse, NavidromeError> {
        self.get("getPlaylists", &[]).await
    }

    async fn get(
        &self,
        endpoint: &str,
        extra: &[(String, String)],
    ) -> Result<QueryResponse, NavidromeError> {
        let path = format!("/rest/{endpoint}.view");
        let mut query = vec![
            ("u".to_string(), self.username.clone()),
            ("t".to_string(), self.token.clone()),
            ("s".to_string(), self.salt.clone()),
            ("v".to_string(), "1.16.1".to_string()),
            ("c".to_string(), self.client_name.clone()),
            ("f".to_string(), "json".to_string()),
        ];
        query.extend(extra.iter().cloned());
        let raw: SubsonicResponse = self.http.get_json_query(&path, &query).await?;
        Ok(QueryResponse {
            value: redact_subsonic(raw.response),
        })
    }
}

fn clamp_size(size: u32) -> Result<u32, NavidromeError> {
    if size == 0 || size > MAX_SIZE {
        return Err(NavidromeError::InvalidParam(format!(
            "size must be between 1 and {MAX_SIZE}"
        )));
    }
    Ok(size)
}

fn redact_subsonic(mut value: serde_json::Value) -> serde_json::Value {
    if let Some(object) = value.as_object_mut() {
        for key in ["salt", "token", "password"] {
            object.remove(key);
        }
    }
    value
}
