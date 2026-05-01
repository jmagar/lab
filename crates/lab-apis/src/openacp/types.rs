//! OpenACP request/response types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Flexible JSON response returned by many OpenACP daemon endpoints.
pub type OpenAcpValue = Value;

/// Request body for creating a new OpenACP session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
}

/// Request body for enqueueing a prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptRequest {
    pub prompt: String,
}

/// Request body for setting session permission bypass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BypassRequest {
    pub enabled: bool,
}

/// Request body for resolving a permission request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResolveRequest {
    #[serde(rename = "permissionId")]
    pub permission_id: String,
    #[serde(rename = "optionId")]
    pub option_id: String,
}

/// Request body for adopting an existing external agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdoptSessionRequest {
    pub agent: String,
    #[serde(rename = "agentSessionId")]
    pub agent_session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
}

/// Request body for patching one safe config value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPatchRequest {
    pub path: String,
    pub value: Value,
}

/// Request body for cleaning up topics by status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TopicsCleanupRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statuses: Option<Vec<String>>,
}

/// Request body for creating a user tunnel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelCreateRequest {
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(rename = "sessionId", skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// Request body for sending a channel notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyRequest {
    pub message: String,
}
