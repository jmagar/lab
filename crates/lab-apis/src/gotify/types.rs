//! Gotify request/response types.
//!
//! Mirrors the Gotify Swagger 2.0 spec. Field names and priorities follow
//! the upstream schema exactly.

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

/// A Gotify client (subscriber that receives messages).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: ClientId,
    pub name: String,
    pub token: String,
}
