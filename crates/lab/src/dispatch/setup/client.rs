//! Path resolution helpers + cached registry views for the `setup`
//! dispatch service.
//!
//! Honors `LAB_HOME` for tests; defaults to `~/.lab/` in production. The
//! registry-derived caches live here so dispatch and secret_mask don't
//! rebuild them on every call.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::OnceLock;

use lab_apis::core::EnvVar;

use crate::registry::{ToolRegistry, build_default_registry, service_meta};

/// Resolve the lab home directory: `$LAB_HOME` if set, else `$HOME/.lab/`.
#[must_use]
pub fn lab_home() -> PathBuf {
    if let Ok(home) = std::env::var("LAB_HOME")
        && !home.is_empty()
    {
        return PathBuf::from(home);
    }
    let base = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(base).join(".lab")
}

#[must_use]
pub fn env_path() -> PathBuf {
    lab_home().join(".env")
}

#[must_use]
pub fn draft_path() -> PathBuf {
    lab_home().join(".env.draft")
}

// ─── cached registry views ──────────────────────────────────────────────

static CACHED_REGISTRY: OnceLock<ToolRegistry> = OnceLock::new();
static CACHED_SECRET_KEYS: OnceLock<HashSet<&'static str>> = OnceLock::new();
static CACHED_ENV_VAR_INDEX: OnceLock<HashMap<&'static str, &'static EnvVar>> = OnceLock::new();

/// Returns the lazy-initialized default registry. Built once per process.
pub fn cached_registry() -> &'static ToolRegistry {
    CACHED_REGISTRY.get_or_init(build_default_registry)
}

/// Returns a `HashSet` of every env var name where the registered
/// `EnvVar.secret == true`. O(1) lookup replaces the per-call registry walk.
pub fn cached_secret_keys() -> &'static HashSet<&'static str> {
    CACHED_SECRET_KEYS.get_or_init(|| {
        let mut keys = HashSet::new();
        for entry in cached_registry().services() {
            if let Some(meta) = service_meta(entry.name) {
                for var in meta.required_env.iter().chain(meta.optional_env.iter()) {
                    if var.secret {
                        keys.insert(var.name);
                    }
                }
            }
        }
        keys
    })
}

/// Returns a `HashMap` from env var name to the registered `EnvVar` declaration.
/// O(1) lookup replaces the per-entry registry rebuild in
/// `validate_against_registry`.
pub fn cached_env_var_index() -> &'static HashMap<&'static str, &'static EnvVar> {
    CACHED_ENV_VAR_INDEX.get_or_init(|| {
        let mut idx = HashMap::new();
        for entry in cached_registry().services() {
            if let Some(meta) = service_meta(entry.name) {
                for var in meta.required_env.iter().chain(meta.optional_env.iter()) {
                    idx.insert(var.name, var);
                }
            }
        }
        idx
    })
}
