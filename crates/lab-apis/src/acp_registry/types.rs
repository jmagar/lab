//! Request/response types for the ACP Agent Registry CDN endpoint.
//!
//! Serde rules:
//! - No `deny_unknown_fields` on `Agent` — the registry adds fields freely.
//! - Use `#[serde(default)]` liberally for optional arrays/fields.
//! - `Distribution` variants use `snake_case` renaming to match JSON keys exactly.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Top-level response
// ---------------------------------------------------------------------------

/// Top-level response envelope from the ACP Registry CDN.
///
/// Endpoint: `GET /registry/v1/latest/registry.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpRegistryResponse {
    /// Schema version, e.g. `"1.0.0"`.
    pub version: String,
    /// All registered ACP agents.
    #[serde(default)]
    pub agents: Vec<Agent>,
    /// Extension entries (reserved for future use; currently empty `[]`).
    #[serde(default)]
    pub extensions: Vec<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Agent
// ---------------------------------------------------------------------------

/// A single ACP-compatible agent entry from the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Unique agent identifier (e.g. `"anthropic/claude-code"`).
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Semver version string.
    pub version: String,
    /// Short description of the agent.
    #[serde(default)]
    pub description: Option<String>,
    /// How this agent is distributed and run.
    pub distribution: Distribution,
    /// Environment variables the agent accepts.
    #[serde(default)]
    pub env: Vec<AgentEnvVar>,
    /// Any additional fields not captured above.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Distribution
// ---------------------------------------------------------------------------

/// Distribution method and assets for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Distribution {
    /// Pre-compiled binary assets keyed by platform triple
    /// (e.g. `"darwin-aarch64"`, `"linux-x86_64"`, `"windows-x86_64"`).
    Binary(HashMap<String, BinaryAsset>),
    /// Run via `npx <package>`.
    Npx(NpxAsset),
    /// Run via `uvx <package>`.
    Uvx(UvxAsset),
}

/// A single platform binary asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryAsset {
    /// URL to download the archive (`.tar.gz` or `.zip`).
    pub archive: String,
    /// Command to run after extraction (e.g. `"./my-agent"`).
    pub cmd: String,
}

/// npm/npx distribution asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpxAsset {
    /// npm package name (e.g. `"@scope/agent"`).
    pub package: String,
    /// Package version (e.g. `"1.2.3"`).
    pub version: String,
}

/// uv/uvx distribution asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UvxAsset {
    /// PyPI package name.
    pub package: String,
    /// Package version (e.g. `"1.2.3"`).
    pub version: String,
}

// ---------------------------------------------------------------------------
// AgentEnvVar
// ---------------------------------------------------------------------------

/// An environment variable declaration from an agent manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEnvVar {
    /// The environment variable name (e.g. `"ANTHROPIC_API_KEY"`).
    pub name: String,
    /// Optional human-readable description.
    #[serde(default)]
    pub description: Option<String>,
    /// Whether this variable is required for the agent to function.
    #[serde(default)]
    pub required: bool,
}
