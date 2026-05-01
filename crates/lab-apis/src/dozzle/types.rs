//! Dozzle request and response types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Health response returned by `server.health`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub reachable: bool,
}

/// Version response returned by `server.version`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionResponse {
    pub version: String,
}

/// Caller-controlled bounds for Dozzle reads that could otherwise run forever
/// or return unbounded history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadLimits {
    pub max_events: usize,
    pub max_lines: usize,
    pub max_bytes: usize,
    pub timeout_ms: u64,
}

impl Default for ReadLimits {
    fn default() -> Self {
        Self {
            max_events: 20,
            max_lines: 1_000,
            max_bytes: 1_048_576,
            timeout_ms: 3_000,
        }
    }
}

impl ReadLimits {
    pub const MAX_EVENTS_CAP: usize = 200;
    pub const MAX_LINES_CAP: usize = 10_000;
    pub const MAX_BYTES_CAP: usize = 10_485_760;
    pub const TIMEOUT_MS_CAP: u64 = 10_000;

    #[must_use]
    pub fn capped(mut self) -> Self {
        self.max_events = self.max_events.clamp(1, Self::MAX_EVENTS_CAP);
        self.max_lines = self.max_lines.clamp(1, Self::MAX_LINES_CAP);
        self.max_bytes = self.max_bytes.clamp(1, Self::MAX_BYTES_CAP);
        self.timeout_ms = self.timeout_ms.clamp(1, Self::TIMEOUT_MS_CAP);
        self
    }
}

/// Metadata returned with bounded reads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitMetadata {
    pub truncated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_kind: Option<String>,
    pub bytes_read: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events_read: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines_read: Option<usize>,
    pub duration_ms: u128,
}

/// Container inventory returned by `containers.list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainersListResponse {
    pub containers: Vec<Value>,
    pub meta: LimitMetadata,
}

/// Parameters for historical Dozzle log fetches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFetchRequest {
    pub host: String,
    pub container_id: String,
    pub stdout: bool,
    pub stderr: bool,
    pub limits: ReadLimits,
}

/// Parsed JSONL log fetch response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFetchResponse {
    pub events: Vec<Value>,
    pub meta: LimitMetadata,
}

/// Plain-text log fetch response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlainLogFetchResponse {
    pub text: String,
    pub meta: LimitMetadata,
}

#[derive(Debug, Clone)]
pub(crate) struct LimitedText {
    pub text: String,
    pub bytes_read: usize,
    pub truncated: bool,
    pub limit_kind: Option<String>,
    pub duration_ms: u128,
}
