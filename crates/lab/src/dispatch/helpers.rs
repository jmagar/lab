//! Shared dispatch helpers used across all service modules.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Component, Path};
use std::sync::Arc;

use lab_apis::core::action::ActionSpec;
use serde_json::Value;

use crate::config::scan_instances;
use crate::dispatch::error::ToolError;

/// Replace the user's home-directory prefix with literal `~` so paths
/// embedded in log events, response bodies, and error messages don't leak
/// the OS username.
///
/// Preserves per-runtime subdirs (`~/.claude/plugins/` vs `~/.codex/plugins/`
/// vs `~/.lab/bin/<agent_id>/` remain distinguishable). Safe on any input:
/// if `HOME` is unset or the path doesn't sit under it, the input is
/// returned unchanged.
///
/// lab-zxx5.27: promoted to shared helpers so `node/` install paths can
/// call it without reaching into a sibling service's private module.
#[must_use]
pub fn redact_home(path: &str) -> String {
    let Some(home) = std::env::var_os("HOME") else {
        return path.to_string();
    };
    let home = home.to_string_lossy();
    let home = home.trim_end_matches('/');
    if home.is_empty() {
        return path.to_string();
    }
    if let Some(rest) = path.strip_prefix(home) {
        let rest = rest.trim_start_matches('/');
        if rest.is_empty() {
            return "~".to_string();
        }
        return format!("~/{rest}");
    }
    path.to_string()
}

/// Reject any path input that contains a `Component::ParentDir` (`..`) segment.
///
/// This is a **lexical** check only. Callers that join the input against a
/// trusted root MUST additionally `canonicalize` + `starts_with(root)` after
/// writing to protect against symlinks escaping the jail (TOCTOU-weak, but
/// strictly better than skipping). Windows UNC / absolute paths are rejected
/// upstream by callers via `Path::is_absolute`.
pub fn reject_path_traversal(rel_path: &str) -> Result<(), ToolError> {
    for component in Path::new(rel_path).components() {
        if matches!(component, Component::ParentDir) {
            return Err(ToolError::InvalidParam {
                message: format!("path traversal rejected: `{rel_path}` contains `..`"),
                param: "path".to_string(),
            });
        }
    }
    Ok(())
}

/// Read an environment variable, returning `None` if absent or empty.
pub fn env_non_empty(name: &str) -> Option<String> {
    ENV_OVERRIDE
        .with(|override_map| {
            override_map
                .borrow()
                .as_ref()
                .and_then(|values| values.get(name).cloned())
        })
        .or_else(|| std::env::var(name).ok())
        .filter(|v| !v.is_empty())
}

thread_local! {
    static ENV_OVERRIDE: RefCell<Option<HashMap<String, String>>> = const { RefCell::new(None) };
}

pub fn with_env_override<T>(values: HashMap<String, String>, f: impl FnOnce() -> T) -> T {
    ENV_OVERRIDE.with(|slot| {
        let previous = slot.replace(Some(values));
        let result = f();
        slot.replace(previous);
        result
    })
}

/// Serialize any `Serialize` value to `serde_json::Value`.
pub fn to_json<T: serde::Serialize>(v: T) -> Result<Value, ToolError> {
    serde_json::to_value(v).map_err(|e| ToolError::Sdk {
        sdk_kind: "decode_error".to_string(),
        message: e.to_string(),
    })
}

/// Extract a required string parameter from a JSON object.
pub fn require_str<'a>(params: &'a Value, key: &str) -> Result<&'a str, ToolError> {
    params
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| ToolError::MissingParam {
            message: format!("missing required parameter `{key}`"),
            param: key.to_string(),
        })
}

/// Extract an optional string parameter from a JSON object.
///
/// Empty strings are rejected as `invalid_param` so callers do not silently
/// treat `instance=` as if the field were absent.
pub fn optional_str<'a>(params: &'a Value, key: &str) -> Result<Option<&'a str>, ToolError> {
    match params.get(key) {
        None => Ok(None),
        Some(v) => {
            let value = v.as_str().ok_or_else(|| ToolError::InvalidParam {
                message: format!("parameter `{key}` must be a string"),
                param: key.to_string(),
            })?;
            if value.is_empty() {
                Err(ToolError::InvalidParam {
                    message: format!("parameter `{key}` must not be empty"),
                    param: key.to_string(),
                })
            } else {
                Ok(Some(value))
            }
        }
    }
}

/// Extract a required integer parameter from a JSON object.
pub fn require_i64(params: &Value, key: &str) -> Result<i64, ToolError> {
    params.get(key).map_or_else(
        || {
            Err(ToolError::MissingParam {
                message: format!("missing required parameter `{key}`"),
                param: key.to_string(),
            })
        },
        |v| {
            v.as_i64().ok_or_else(|| ToolError::InvalidParam {
                message: format!("parameter `{key}` must be an integer"),
                param: key.to_string(),
            })
        },
    )
}

/// Extract an optional non-negative integer parameter from a JSON object.
pub fn optional_u32(params: &Value, key: &str) -> Result<Option<u32>, ToolError> {
    params.get(key).map_or(Ok(None), |v| {
        v.as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .map(Some)
            .ok_or_else(|| ToolError::InvalidParam {
                message: format!("parameter `{key}` must be a non-negative integer"),
                param: key.to_string(),
            })
    })
}

/// Extract an optional non-negative integer parameter and enforce an upper bound.
pub fn optional_u32_max(params: &Value, key: &str, max: u32) -> Result<Option<u32>, ToolError> {
    match optional_u32(params, key)? {
        Some(n) if n > max => Err(ToolError::InvalidParam {
            message: format!("parameter `{key}` must be between 0 and {max}"),
            param: key.to_string(),
        }),
        other => Ok(other),
    }
}

/// Clone a JSON object, stripping the given keys.
pub fn object_without(params: &Value, strip: &[&str]) -> Value {
    let mut map = params.as_object().cloned().unwrap_or_default();
    for key in strip {
        map.remove(*key);
    }
    Value::Object(map)
}

/// Build the standard `help` response payload for a service.
///
/// Produces the canonical `{ service, actions: [...] }` shape returned by every
/// service dispatcher when `action == "help"`.
pub fn help_payload(service: &str, actions: &[ActionSpec]) -> Value {
    serde_json::json!({
        "service": service,
        "actions": actions.iter().map(|a| serde_json::json!({
            "name": a.name,
            "description": a.description,
            "destructive": a.destructive,
            "returns": a.returns,
            "params": a.params.iter().map(|p| serde_json::json!({
                "name": p.name,
                "type": p.ty,
                "required": p.required,
                "description": p.description,
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })
}

/// Return the schema for one named action.
///
/// Used to implement the `"schema"` built-in action in every service dispatcher.
/// Returns `ToolError::UnknownAction` if `action_name` is not in `actions`.
pub fn action_schema(actions: &[ActionSpec], action_name: &str) -> Result<Value, ToolError> {
    let spec = actions
        .iter()
        .find(|a| a.name == action_name)
        .ok_or_else(|| ToolError::UnknownAction {
            message: format!("no schema for unknown action `{action_name}`"),
            valid: actions.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        })?;
    Ok(serde_json::json!({
        "action": spec.name,
        "description": spec.description,
        "destructive": spec.destructive,
        "returns": spec.returns,
        "params": spec.params.iter().map(|p| serde_json::json!({
            "name": p.name,
            "type": p.ty,
            "required": p.required,
            "description": p.description,
        })).collect::<Vec<_>>(),
    }))
}

/// Generic pool of named service clients built once at first access.
///
/// `InstancePool<C>` consolidates the three-static / two-static `OnceLock`
/// patterns that were previously duplicated verbatim across every
/// multi-instance service (`unifi`, `unraid`, …).
///
/// # Usage
///
/// ```rust,ignore
/// static POOL: OnceLock<InstancePool<MyClient>> = OnceLock::new();
///
/// fn pool() -> &'static InstancePool<MyClient> {
///     POOL.get_or_init(|| {
///         InstancePool::build("MY_SERVICE", |url, key| {
///             MyClient::new(&url, Auth::ApiKey { header: "X-Api-Key".into(), key }).ok()
///         })
///     })
/// }
/// ```
pub struct InstancePool<C> {
    clients: HashMap<String, Arc<C>>,
    all_labels: Vec<String>,
}

impl<C: Send + Sync + 'static> InstancePool<C> {
    /// Scan `prefix` instances from the environment and construct clients.
    ///
    /// `build(url, key) -> Option<C>`: return `None` to skip a label (e.g.,
    /// when client construction fails). A `tracing::warn!` inside the closure
    /// is recommended but not required.
    pub fn build<F>(prefix: &str, build: F) -> Self
    where
        F: Fn(String, String) -> Option<C>,
    {
        let mut clients = HashMap::new();
        let mut all_labels = Vec::new();
        for (label, _vars) in scan_instances(prefix) {
            all_labels.push(label.clone());
            let (url_key, key_key) = instance_env_keys(prefix, &label);
            if let (Some(url), Some(key)) = (env_non_empty(&url_key), env_non_empty(&key_key))
                && let Some(client) = build(url, key)
            {
                clients.insert(label, Arc::new(client));
            }
        }
        Self {
            clients,
            all_labels,
        }
    }

    /// Look up a client by exact (already-normalised) label.
    #[allow(dead_code)]
    pub fn get(&self, label: &str) -> Option<&Arc<C>> {
        self.clients.get(label)
    }

    /// All instance labels discovered in the environment (including those with
    /// broken or incomplete configuration).
    #[allow(dead_code)]
    pub fn all_labels(&self) -> &[String] {
        &self.all_labels
    }

    /// Resolve a client by optional instance label.
    ///
    /// - `None` — use the default instance (equivalent to `Some("default")`).
    /// - `Some(label)` — look up the named instance.
    ///
    /// # Errors
    ///
    /// - `InvalidParam` — `label` is empty.
    /// - `Sdk { sdk_kind: "internal_error" }` — the label was discovered in the
    ///   environment but its URL or API key is missing or invalid.
    /// - `UnknownInstance` — the label was never found in the environment.
    pub fn resolve(&self, label: Option<&str>) -> Result<Arc<C>, ToolError> {
        let label = label.unwrap_or("default").trim();
        if label.is_empty() {
            return Err(ToolError::InvalidParam {
                message: "parameter `instance` must not be empty".to_string(),
                param: "instance".to_string(),
            });
        }
        let label = label.to_ascii_lowercase();
        if let Some(client) = self.clients.get(&label) {
            return Ok(client.clone());
        }
        // Label was discovered in env but the config is incomplete.
        if self.all_labels.iter().any(|l| l == &label) {
            return Err(ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message: format!(
                    "instance `{label}` is configured but URL or API key is missing or invalid"
                ),
            });
        }
        let mut valid: Vec<String> = self.clients.keys().cloned().collect();
        valid.sort();
        Err(ToolError::UnknownInstance {
            message: format!("unknown instance `{label}`"),
            valid,
        })
    }
}

/// Build the env var key names for a service prefix and instance label.
///
/// - default instance: `PREFIX_URL` / `PREFIX_API_KEY`
/// - named instance:   `PREFIX_LABEL_URL` / `PREFIX_LABEL_API_KEY`
pub fn instance_env_keys(prefix: &str, label: &str) -> (String, String) {
    if label == "default" {
        (format!("{prefix}_URL"), format!("{prefix}_API_KEY"))
    } else {
        let upper = label.to_ascii_uppercase();
        (
            format!("{prefix}_{upper}_URL"),
            format!("{prefix}_{upper}_API_KEY"),
        )
    }
}

/// Create a database file with Unix 0600 permissions if it does not already exist.
///
/// This is a no-op when the file already exists (uses `create_new`, so an
/// `AlreadyExists` error is silently swallowed). On platforms that do not
/// support `OpenOptionsExt`, this function is not compiled and callers must
/// ensure appropriate permissions by other means.
///
/// # Security
///
/// Must be called **before** opening the file via the connection pool so that
/// the creation and permission assignment happen atomically. Subsequent opens
/// by the pool do not change permissions.
#[cfg(unix)]
#[allow(dead_code)]
pub fn create_db_file_0600(path: &std::path::PathBuf) {
    use std::os::unix::fs::OpenOptionsExt;
    // Only set mode on creation; if the file already exists, leave perms alone.
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true) // fails silently if file exists — that's fine
        .mode(0o600)
        .open(path)
        .ok();
}

/// Build a request body from params.
///
/// If `params` contains a `payload` or `body` key, that value is used as the
/// body (parsed from a JSON string if necessary). Otherwise the entire params
/// object is used, minus the `strip` keys.
pub fn body_from_params(params: &Value, strip: &[&str]) -> Value {
    for key in ["payload", "body"] {
        if let Some(value) = params.get(key) {
            match value {
                Value::Object(map) => return Value::Object(map.clone()),
                Value::String(raw) => {
                    if let Ok(parsed) = serde_json::from_str(raw) {
                        return parsed;
                    }
                }
                _ => {}
            }
        }
    }
    object_without(params, strip)
}
