//! Client construction helpers for the `unraid` dispatch layer.

use std::sync::{Arc, OnceLock};

use lab_apis::core::Auth;
use lab_apis::unraid::UnraidClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{InstancePool, env_non_empty};

/// Strip trailing `/graphql` and surrounding slashes from an Unraid URL.
///
/// The URL stored in env can be either the bare host (`https://host:31337`)
/// or the full endpoint (`https://host:31337/graphql`) — the client always
/// appends `/graphql` itself, so we normalise to the bare host form.
fn normalize_unraid_url(raw: &str) -> String {
    raw.trim_end_matches('/')
        .trim_end_matches("graphql")
        .trim_end_matches('/')
        .to_string()
}

static POOL: OnceLock<InstancePool<UnraidClient>> = OnceLock::new();

/// Return (or lazily build) the `Unraid` instance pool.
///
/// Built once by scanning env vars at first call. Subsequent calls are
/// lock-free reads.
fn pool() -> &'static InstancePool<UnraidClient> {
    POOL.get_or_init(|| {
        InstancePool::build("UNRAID", |url, key| {
            let url = normalize_unraid_url(&url);
            UnraidClient::new(
                &url,
                Auth::ApiKey {
                    // Unraid's GraphQL API requires "X-API-Key" (all-caps KEY) per its
                    // API spec — intentional deviation from the dispatch template which
                    // uses "X-Api-Key". HTTP headers are case-insensitive on the wire,
                    // but Unraid's server validates the exact name.
                    header: "X-API-Key".into(),
                    key,
                },
            )
            .ok()
        })
    })
}

/// Build an `UnraidClient` from the default-instance environment variables.
///
/// Reads `UNRAID_URL` and `UNRAID_API_KEY`. Returns `None` if either is
/// absent or empty. Called by `AppState::ServiceClients::from_env()` at
/// startup — must be pure (no side effects, no logging).
pub fn client_from_env() -> Option<UnraidClient> {
    let raw_url = env_non_empty("UNRAID_URL")?;
    let url = normalize_unraid_url(&raw_url);
    let key = env_non_empty("UNRAID_API_KEY")?;
    UnraidClient::new(
        &url,
        Auth::ApiKey {
            // Unraid's GraphQL API requires "X-API-Key" (all-caps KEY) per its
            // API spec — intentional deviation from the dispatch template which
            // uses "X-Api-Key". HTTP headers are case-insensitive on the wire,
            // but Unraid's server validates the exact name.
            header: "X-API-Key".into(),
            key,
        },
    )
    .ok()
}

/// Resolve an `Unraid` client by optional instance label.
///
/// - `None` — use the default instance (equivalent to `Some("default")`).
/// - `Some(label)` — look up the named instance in the module-level cache.
///
/// Returns an `Arc`-wrapped client from the module-level cache — no new
/// `reqwest::Client` is created after the first call.
///
/// # Errors
///
/// - `InvalidParam` — `label` is empty.
/// - `Sdk { sdk_kind: "internal_error" }` — the label was discovered in the
///   environment but its URL or API key is missing or invalid.
/// - `UnknownInstance` — the label was never found in the environment at all.
pub fn client_from_instance(label: Option<&str>) -> Result<Arc<UnraidClient>, ToolError> {
    pool().resolve(label)
}

/// Structured error for callers that hold a pre-built `Option<UnraidClient>`.
/// The API handler calls this directly instead of re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "UNRAID_URL or UNRAID_API_KEY not configured".to_string(),
    }
}

/// Return a client or a structured `internal_error` if not configured.
///
/// Used by MCP and CLI dispatch where `AppState` is not available.
#[allow(dead_code)]
pub fn require_client() -> Result<UnraidClient, ToolError> {
    client_from_env().ok_or_else(not_configured_error)
}
