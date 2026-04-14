//! Gotify request/response types.
//!
//! Mirrors the Gotify Swagger 2.0 spec. Field names follow the upstream schema.

use serde::{Deserialize, Serialize};

/// Newtype wrapper around a Gotify message id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MessageId(pub u64);

/// Newtype wrapper around a Gotify application id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ApplicationId(pub u64);

/// Newtype wrapper around a Gotify client id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClientId(pub u64);

/// Newtype wrapper around a Gotify user id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(pub u64);

/// A message posted to Gotify.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub appid: ApplicationId,
    pub message: String,
    pub title: Option<String>,
    /// 0–10; higher priorities bypass Do Not Disturb on clients.
    pub priority: Option<i32>,
    pub date: String,
    #[serde(default)]
    pub extras: serde_json::Value,
}

/// Request body for `POST /message`.
#[derive(Debug, Clone, Serialize)]
pub struct SendMessage {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extras: Option<serde_json::Value>,
}

/// Paging metadata from Gotify list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paging {
    pub size: i64,
    pub page: i64,
    pub total_page: i64,
    pub since: Option<i64>,
    pub limit: i64,
}

/// Paginated message list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedMessages {
    pub paging: Paging,
    pub messages: Vec<Message>,
}

/// Server health status from `GET /health`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    /// Overall health — `"green"` when healthy.
    pub health: String,
    /// Database health — `"green"` when healthy.
    pub database: String,
}

/// A Gotify application (channel for sending messages).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub id: ApplicationId,
    pub name: String,
    pub description: String,
    pub token: String,
    #[serde(default)]
    pub internal: bool,
    pub image: Option<String>,
}

/// Request body for creating or updating an application.
#[derive(Debug, Clone, Serialize)]
pub struct ApplicationParams {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A Gotify client (subscriber that receives messages).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: ClientId,
    pub name: String,
    pub token: String,
}

/// Request body for creating or updating a client.
#[derive(Debug, Clone, Serialize)]
pub struct ClientParams {
    pub name: String,
}

/// Newtype wrapper around a Gotify plugin id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PluginId(pub u64);

/// A Gotify server plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: PluginId,
    pub name: String,
    pub module_path: Option<String>,
    pub enabled: bool,
    pub capabilities: Option<Vec<String>>,
    pub license: Option<String>,
    pub description: Option<String>,
    pub website: Option<String>,
}

/// Request body for creating a Gotify user.
#[derive(Debug, Clone, Serialize)]
pub struct UserCreate {
    pub name: String,
    pub pass: String,
    pub admin: bool,
}

/// Server version information from `GET /version`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerVersion {
    pub version: String,
    pub commit: String,
    #[serde(rename = "buildDate")]
    pub build_date: String,
}
