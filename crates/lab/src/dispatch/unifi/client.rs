use std::sync::{Arc, OnceLock};

use lab_apis::core::Auth;
use lab_apis::unifi::UnifiClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{InstancePool, env_non_empty};

static POOL: OnceLock<InstancePool<UnifiClient>> = OnceLock::new();

/// Return (or lazily build) the `UniFi` instance pool.
///
/// Built once by scanning env vars at first call. Subsequent calls are
/// lock-free reads.
fn pool() -> &'static InstancePool<UnifiClient> {
    POOL.get_or_init(|| {
        InstancePool::build("UNIFI", |url, key| {
            UnifiClient::new(
                &url,
                Auth::ApiKey {
                    header: "X-API-KEY".into(),
                    key,
                },
            )
            .map_err(|e| tracing::warn!(error = %e, "unifi client construction failed"))
            .ok()
        })
    })
}

/// Build a `UniFi` client from the default-instance env vars.
pub fn client_from_env() -> Option<UnifiClient> {
    let url = env_non_empty("UNIFI_URL")?;
    let key = env_non_empty("UNIFI_API_KEY")?;
    UnifiClient::new(
        &url,
        Auth::ApiKey {
            header: "X-API-KEY".into(),
            key,
        },
    )
    .map_err(|e| tracing::warn!(error = %e, url, "unifi client construction failed"))
    .ok()
}

/// Resolve a `UniFi` client by optional instance label.
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
pub fn client_from_instance(label: Option<&str>) -> Result<Arc<UnifiClient>, ToolError> {
    pool().resolve(label)
}

#[allow(dead_code)]
pub fn require_client() -> Result<UnifiClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "UNIFI_URL or UNIFI_API_KEY not configured".to_string(),
    })
}
