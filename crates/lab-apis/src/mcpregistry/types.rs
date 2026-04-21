//! Request/response types for the MCP Registry v0.1 API.
//!
//! Serde rules:
//! - No `deny_unknown_fields` — the registry adds fields freely.
//! - Nullable arrays use `#[serde(default)]` to treat JSON null as empty.
//! - Reserved-word fields use descriptive Rust names with `#[serde(rename = "type")]`.
//! - Dotted/slashed JSON keys use `#[serde(rename = "...")]` directly.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Query params
// ---------------------------------------------------------------------------

/// Parameters for `list_servers` (GET /v0.1/servers).
#[derive(Debug, Clone, Default)]
pub struct ListServersParams {
    /// Substring search on server name or description.
    pub search: Option<String>,
    /// Pagination cursor from a previous response `metadata.next_cursor`.
    pub cursor: Option<String>,
    /// Page size (1–100; server default: 20).
    pub limit: Option<u32>,
    /// Filter by version string or `"latest"`.
    pub version: Option<String>,
    /// RFC3339 timestamp filter — only servers updated after this time.
    pub updated_since: Option<String>,
}

impl ListServersParams {
    /// Convert to wire query-parameter pairs.
    pub fn to_query_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        if let Some(v) = &self.search {
            pairs.push(("search".to_string(), v.clone()));
        }
        if let Some(v) = &self.cursor {
            pairs.push(("cursor".to_string(), v.clone()));
        }
        if let Some(v) = self.limit {
            pairs.push(("limit".to_string(), v.to_string()));
        }
        if let Some(v) = &self.version {
            pairs.push(("version".to_string(), v.clone()));
        }
        if let Some(v) = &self.updated_since {
            pairs.push(("updatedSince".to_string(), v.clone()));
        }
        pairs
    }
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// Paginated list of MCP servers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerListResponse {
    /// Page of server entries. JSON null is treated as an empty list.
    #[serde(default)]
    pub servers: Vec<ServerResponse>,
    /// Pagination metadata.
    pub metadata: Metadata,
}

/// Pagination metadata attached to list responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// Number of items in the current page.
    pub count: i64,
    /// Cursor for retrieving the next page; absent when on the last page.
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}

/// A single server entry from the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerResponse {
    /// Server configuration and metadata.
    pub server: ServerJSON,
    /// Registry-managed metadata (may be absent in some response contexts).
    #[serde(rename = "_meta")]
    pub meta: Option<ResponseMeta>,
}

/// Registry-managed per-response metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    /// Official registry extensions.
    #[serde(rename = "io.modelcontextprotocol.registry/official")]
    pub official: Option<RegistryExtensions>,
}

/// Registry lifecycle extensions attached to a server version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryExtensions {
    /// Whether this is the latest published version.
    #[serde(rename = "isLatest")]
    pub is_latest: bool,
    /// Timestamp when the server was first published.
    #[serde(rename = "publishedAt")]
    pub published_at: String,
    /// Lifecycle status: `"active"`, `"deprecated"`, or `"deleted"`.
    pub status: String,
    /// Timestamp of the most recent status change.
    #[serde(rename = "statusChangedAt")]
    pub status_changed_at: String,
    /// Optional human-readable reason for the status change.
    #[serde(rename = "statusMessage")]
    pub status_message: Option<String>,
    /// Timestamp of the most recent metadata update.
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
}

/// Full server configuration from `server.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerJSON {
    /// JSON Schema URI for this server.json format.
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    /// Server name in reverse-DNS format (e.g. `io.github.user/weather`).
    pub name: String,
    /// Optional human-readable display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Human-readable explanation of server functionality.
    pub description: String,
    /// Version string (ideally semver).
    pub version: String,
    /// Package configurations (local/stdio transports).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<Package>,
    /// Remote transport configurations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remotes: Vec<Transport>,
    /// Source code repository metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<Repository>,
    /// Display icons.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub icons: Vec<Icon>,
    /// Homepage or project website URL.
    #[serde(rename = "websiteUrl", skip_serializing_if = "Option::is_none")]
    pub website_url: Option<String>,
}

/// Transport configuration (used by both `packages[].transport` and `remotes`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transport {
    /// Transport type: `"stdio"`, `"streamable-http"`, or `"sse"`.
    #[serde(rename = "type")]
    pub transport_type: String,
    /// URL for HTTP-based transports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// HTTP headers for HTTP-based transports.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub headers: Vec<serde_json::Value>,
    /// URL template variables for remote transports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<serde_json::Value>,
}

/// A single environment variable declaration from the registry.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnvironmentVariable {
    /// Variable name (e.g. `GITHUB_TOKEN`).
    pub name: String,
    /// Human-readable description shown in install dialogs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether a value must be supplied before the server can run.
    #[serde(rename = "isRequired", default)]
    pub is_required: bool,
    /// Whether the value should be treated as a secret (masked in UIs, written to `.env`).
    #[serde(rename = "isSecret", default)]
    pub is_secret: bool,
    /// Default value used when the user provides none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    /// Enumerated allowed values.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub choices: Vec<String>,
    /// Placeholder text for the install dialog input field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    /// Semantic format hint (e.g. `"token"`, `"url"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

/// Package distribution configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    /// Registry type: `"npm"`, `"pypi"`, `"oci"`, `"nuget"`, `"mcpb"`.
    #[serde(rename = "registryType")]
    pub registry_type: String,
    /// Package identifier or download URL.
    pub identifier: String,
    /// Specific version string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Transport configuration for this package.
    pub transport: Transport,
    /// Runtime hint (e.g. `"npx"`).
    #[serde(rename = "runtimeHint", skip_serializing_if = "Option::is_none")]
    pub runtime_hint: Option<String>,
    /// Arguments passed to the runtime command.
    #[serde(
        rename = "runtimeArguments",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub runtime_arguments: Vec<serde_json::Value>,
    /// Arguments passed to the package binary.
    #[serde(
        rename = "packageArguments",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub package_arguments: Vec<serde_json::Value>,
    /// Environment variables for the package runtime.
    #[serde(
        rename = "environmentVariables",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub environment_variables: Vec<EnvironmentVariable>,
    /// SHA-256 hash of the package file for integrity verification.
    #[serde(rename = "fileSha256", skip_serializing_if = "Option::is_none")]
    pub file_sha256: Option<String>,
    /// Base URL of the package registry.
    #[serde(rename = "registryBaseUrl", skip_serializing_if = "Option::is_none")]
    pub registry_base_url: Option<String>,
}

/// Source repository metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Hosting service identifier (e.g. `"github"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Repository URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Stable repository identifier from the hosting service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Relative path to the server within a monorepo.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subfolder: Option<String>,
}

/// Display icon metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Icon {
    /// Icon URL (HTTPS required).
    pub src: String,
    /// MIME type override.
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Size specifiers (e.g. `"48x48"`, `"any"`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sizes: Vec<String>,
    /// Theme hint: `"light"` or `"dark"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Result of POST /v0.1/validate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the server JSON is valid.
    pub valid: bool,
    /// Validation issues found. JSON null is treated as empty.
    #[serde(default)]
    pub issues: Vec<ValidationIssue>,
}

/// A single validation issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Human-readable description of the issue.
    pub message: String,
    /// JSON path where the issue was found.
    pub path: String,
    /// Severity level (e.g. `"error"`, `"warning"`).
    pub severity: String,
    /// Issue type/code.
    #[serde(rename = "type")]
    pub issue_type: String,
    /// Reference to documentation or specification.
    pub reference: String,
}

// ---------------------------------------------------------------------------
// Health
// ---------------------------------------------------------------------------

/// Response from GET /v0.1/health.
#[derive(Debug, Deserialize)]
pub struct HealthBody {
    /// Health status string (e.g. `"ok"`).
    pub status: String,
}
