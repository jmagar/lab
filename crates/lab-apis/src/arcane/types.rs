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

// ── Projects ──────────────────────────────────────────────────────────────────

/// A Compose/Docker project as reported by Arcane.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub status: Option<String>,
    pub environment_id: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Generic project action result (up/down/redeploy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectActionResult {
    pub success: Option<bool>,
    pub message: Option<String>,
}

// ── Volumes ───────────────────────────────────────────────────────────────────

/// A Docker volume as reported by Arcane.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
    pub name: String,
    pub driver: Option<String>,
    pub mountpoint: Option<String>,
    pub scope: Option<String>,
    pub created_at: Option<String>,
    pub labels: Option<serde_json::Value>,
    pub options: Option<serde_json::Value>,
}

/// Result returned by prune operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PruneResult {
    pub volumes_deleted: Option<Vec<String>>,
    pub space_reclaimed: Option<i64>,
}

// ── Images ────────────────────────────────────────────────────────────────────

/// A Docker image as reported by Arcane.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub id: String,
    pub repo_tags: Option<Vec<String>>,
    pub repo_digests: Option<Vec<String>>,
    pub size: Option<i64>,
    pub created: Option<i64>,
    pub labels: Option<serde_json::Value>,
}

/// Result returned by image pull operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePullResult {
    pub success: Option<bool>,
    pub message: Option<String>,
}

/// Result returned by image prune operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagePruneResult {
    pub images_deleted: Option<Vec<serde_json::Value>>,
    pub space_reclaimed: Option<i64>,
}

/// Summary of available image updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageUpdateSummary {
    pub updates_available: Option<i32>,
    pub images: Option<Vec<serde_json::Value>>,
}
