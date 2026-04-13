//! Arcane request/response types.

use serde::{Deserialize, Serialize};

// ── Generic API envelope ──────────────────────────────────────────────────────

/// Generic Arcane API response envelope with a single data field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
}

/// Generic Arcane paginated response envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub success: bool,
    pub data: Option<Vec<T>>,
}

// ── Health ────────────────────────────────────────────────────────────────────

/// Health response from `GET /health`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
}

// ── Environments ──────────────────────────────────────────────────────────────

/// A Docker host environment registered with Arcane.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub id: String,
    pub api_url: String,
    pub status: String,
    pub enabled: bool,
    pub is_edge: bool,
    pub name: Option<String>,
    pub connected: Option<bool>,
    pub connected_at: Option<String>,
    pub last_heartbeat: Option<String>,
}

// ── Containers ────────────────────────────────────────────────────────────────

/// A Docker container summary as reported by Arcane.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Container {
    pub id: String,
    pub names: Vec<String>,
    pub image: String,
    pub image_id: String,
    pub command: String,
    pub created: i64,
    pub state: String,
    pub status: String,
    pub labels: Option<serde_json::Value>,
    pub ports: Option<serde_json::Value>,
}

/// Container action result data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerActionResult {
    pub success: Option<bool>,
    pub message: Option<String>,
}
