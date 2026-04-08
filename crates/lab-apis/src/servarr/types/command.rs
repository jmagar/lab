//! Command (async job) types.

use serde::{Deserialize, Serialize};

/// Newtype wrapper around a command id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CommandId(pub i64);

/// Lifecycle status for an async command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CommandStatus {
    Queued,
    Started,
    Completed,
    Failed,
    Aborted,
}

/// A queued / running / completed *arr command.
///
/// Mirrors `CommandResource` from the Radarr v3 / Sonarr v3 OpenAPI specs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub id: CommandId,
    pub name: String,
    pub status: CommandStatus,
    #[serde(default)]
    pub queued: Option<String>,
    #[serde(default)]
    pub started: Option<String>,
    #[serde(default)]
    pub ended: Option<String>,
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}
