//! ACP types for the `lab` binary crate.
//!
//! Canonical public types now live in `lab_apis::acp::types` under the `Acp*`
//! prefix. This file re-exports them for convenience and retains the legacy
//! `Bridge*` types that `runtime.rs` and `persistence.rs` still use
//! (migration handled by beads 3 and 4).

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Canonical re-exports from lab-apis
// ---------------------------------------------------------------------------

pub use lab_apis::acp::{
    AcpContentBlock, AcpError, AcpEvent, AcpPermissionOption, AcpProviderHealth, AcpSessionState,
    AcpSessionSummary, PersistenceError,
};

// ---------------------------------------------------------------------------
// Legacy Bridge* types — still used by runtime.rs and persistence.rs
// (do not remove until beads 3+4 migrate those files)
// ---------------------------------------------------------------------------

pub type AcpProviderKind = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeSessionStatus {
    Idle,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl BridgeSessionStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderHealth {
    pub provider: AcpProviderKind,
    pub ready: bool,
    pub command: String,
    pub args: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeSessionSummary {
    pub id: String,
    pub provider_session_id: String,
    pub provider: AcpProviderKind,
    pub title: String,
    pub cwd: String,
    pub created_at: String,
    pub updated_at: String,
    pub status: String,
    pub agent_name: String,
    pub agent_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resumable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgePermissionOption {
    pub option_id: String,
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeEvent {
    pub id: String,
    pub seq: u64,
    pub session_id: String,
    pub provider: AcpProviderKind,
    pub kind: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_input: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_output: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_content: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locations: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_options: Option<Vec<BridgePermissionOption>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_selection: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_info: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commands: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_mode: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_update: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingBridgeEvent {
    pub session_id: String,
    pub provider: AcpProviderKind,
    pub kind: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_input: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_output: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_content: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locations: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_options: Option<Vec<BridgePermissionOption>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_selection: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_info: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commands: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_mode: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_update: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<Value>,
}

impl PendingBridgeEvent {
    #[must_use]
    pub fn new(
        session_id: impl Into<String>,
        provider: impl Into<String>,
        kind: impl Into<String>,
    ) -> Self {
        Self {
            session_id: session_id.into(),
            provider: provider.into(),
            kind: kind.into(),
            created_at: jiff::Timestamp::now().to_string(),
            role: None,
            message_id: None,
            text: None,
            title: None,
            status: None,
            tool_call_id: None,
            tool_kind: None,
            raw_input: None,
            raw_output: None,
            tool_content: None,
            locations: None,
            plan: None,
            permission_options: None,
            permission_selection: None,
            session_info: None,
            usage: None,
            commands: None,
            current_mode: None,
            config_update: None,
            prompt_stop_reason: None,
            raw: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StartSessionInput {
    pub cwd: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StartSessionResult {
    pub provider_session_id: String,
    pub agent_name: String,
    pub agent_version: String,
}
