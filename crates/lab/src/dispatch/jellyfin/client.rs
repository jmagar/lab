use std::sync::{Arc, OnceLock};

use lab_apis::jellyfin::JellyfinClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{InstancePool, env_non_empty};

static POOL: OnceLock<InstancePool<JellyfinClient>> = OnceLock::new();

fn build_client(url: &str, key: &str) -> Result<JellyfinClient, lab_apis::jellyfin::JellyfinError> {
    JellyfinClient::new(url, JellyfinClient::auth_from_api_key(key))
}

fn pool() -> &'static InstancePool<JellyfinClient> {
    POOL.get_or_init(|| {
        InstancePool::build("JELLYFIN", |url, key| {
            build_client(&url, &key)
                .map_err(|e| tracing::warn!(error = %e, "jellyfin client construction failed"))
                .ok()
        })
    })
}

/// Build a Jellyfin client from the default-instance env vars.
pub fn client_from_env() -> Option<JellyfinClient> {
    let url = env_non_empty("JELLYFIN_URL")?;
    let key = env_non_empty("JELLYFIN_API_KEY")?;
    build_client(&url, &key).ok()
}

/// Resolve a Jellyfin client by optional instance label.
pub fn client_from_instance(label: Option<&str>) -> Result<Arc<JellyfinClient>, ToolError> {
    pool().resolve(label)
}

/// Structured error returned when Jellyfin config is absent.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "JELLYFIN_URL and JELLYFIN_API_KEY not configured".into(),
    }
}
