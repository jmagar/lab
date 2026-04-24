//! Dispatch for `agent.*` actions within the marketplace service.
//!
//! Routes ACP agent discovery and install/uninstall operations to the
//! `lab-apis::acp_registry` SDK and the local provider config at
//! `~/.lab/acp-providers.json`.
//!
//! Scope notes (lab-zxx5.3):
//! - Only `"local"` device installs are supported in this build. Remote
//!   device installs return a per-device `not_implemented` error envelope —
//!   fleet RPC plumbing for `agent.install` lives in a follow-up bead.
//! - Binary distributions are not yet downloaded/extracted. Only `npx` and
//!   `uvx` distributions write a usable provider config entry. Binary
//!   distributions return `not_implemented` per device.
//! - SHA-256 verification is not performed because `BinaryAsset` in the SDK
//!   does not expose a `sha256` field. When binary install is implemented in
//!   a follow-up, `BinaryAsset.extra` (via `Agent.extra`) can supply it.

use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};
use crate::dispatch::marketplace::acp_catalog::ACP_ACTIONS;
use crate::dispatch::marketplace::acp_client;

#[cfg(feature = "acp_registry")]
use lab_apis::acp_registry::client::AcpRegistryClient;
#[cfg(feature = "acp_registry")]
use lab_apis::acp_registry::types::{Agent, Distribution};

/// Dispatch an `agent.*` action using a freshly constructed client.
pub async fn dispatch_acp(action: &str, params: Value) -> Result<Value, ToolError> {
    // help/schema are universal and feature-independent.
    match action {
        "help" => return Ok(help_payload("marketplace", ACP_ACTIONS)),
        "schema" => {
            let action_name = require_str(&params, "action")?;
            return action_schema(ACP_ACTIONS, action_name);
        }
        _ => {}
    }

    #[cfg(feature = "acp_registry")]
    {
        let client = acp_client::require_acp_client()?;
        dispatch_acp_with_client(&client, action, params).await
    }
    #[cfg(not(feature = "acp_registry"))]
    {
        let _ = (action, params);
        Err(acp_client::require_acp_client().unwrap_err())
    }
}

/// Dispatch an `agent.*` action with a pre-built client (used by API handlers).
#[cfg(feature = "acp_registry")]
pub async fn dispatch_acp_with_client(
    client: &AcpRegistryClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "agent.list" => {
            let agents = client.list_agents().await?;
            to_json(agents)
        }
        "agent.get" => {
            let id = require_str(&params, "id")?.to_string();
            let agent = client.get_agent(&id).await?.ok_or_else(|| ToolError::Sdk {
                sdk_kind: "not_found".to_string(),
                message: format!("agent `{id}` not found in registry"),
            })?;
            to_json(agent)
        }
        "agent.install" => dispatch_install(client, &params).await,
        "agent.uninstall" => {
            let id = require_str(&params, "id")?.to_string();
            dispatch_uninstall(&id)
        }
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action `marketplace.{unknown}`"),
            valid: ACP_ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

// ---------------------------------------------------------------------------
// agent.install
// ---------------------------------------------------------------------------

#[cfg(feature = "acp_registry")]
async fn dispatch_install(
    client: &AcpRegistryClient,
    params: &Value,
) -> Result<Value, ToolError> {
    let id = require_str(params, "id")?.to_string();

    let device_ids: Vec<String> = match params.get("device_ids") {
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        Some(_) => {
            return Err(ToolError::InvalidParam {
                message: "`device_ids` must be an array of strings".to_string(),
                param: "device_ids".to_string(),
            });
        }
        None => {
            return Err(ToolError::MissingParam {
                message: "missing required parameter `device_ids`".to_string(),
                param: "device_ids".to_string(),
            });
        }
    };

    if device_ids.is_empty() {
        return Err(ToolError::InvalidParam {
            message: "`device_ids` must not be empty".to_string(),
            param: "device_ids".to_string(),
        });
    }

    let platform_override = params
        .get("platform")
        .and_then(Value::as_str)
        .map(str::to_string);

    let agent = client.get_agent(&id).await?.ok_or_else(|| ToolError::Sdk {
        sdk_kind: "not_found".to_string(),
        message: format!("agent `{id}` not found in registry"),
    })?;

    let mut results = Vec::with_capacity(device_ids.len());
    for device_id in &device_ids {
        let outcome = if is_local_device(device_id) {
            install_local(&agent, platform_override.as_deref())
        } else {
            // Remote install via fleet RPC is deferred — see module-level note.
            Err(ToolError::Sdk {
                sdk_kind: "not_implemented".to_string(),
                message: format!(
                    "remote install on device `{device_id}` is not yet wired (deferred to follow-up bead)"
                ),
            })
        };
        match outcome {
            Ok(value) => results.push(serde_json::json!({
                "device_id": device_id,
                "ok": true,
                "result": value,
            })),
            Err(e) => results.push(serde_json::json!({
                "device_id": device_id,
                "ok": false,
                "error": serde_json::to_value(&e).unwrap_or(Value::Null),
            })),
        }
    }

    Ok(serde_json::json!({
        "agent_id": id,
        "results": results,
    }))
}

fn is_local_device(device_id: &str) -> bool {
    if device_id.eq_ignore_ascii_case("local") {
        return true;
    }
    if let Ok(host) = std::env::var("HOSTNAME")
        && !host.is_empty()
        && device_id.eq_ignore_ascii_case(&host)
    {
        return true;
    }
    false
}

#[cfg(feature = "acp_registry")]
fn install_local(agent: &Agent, platform_override: Option<&str>) -> Result<Value, ToolError> {
    let (distribution_kind, command) = match &agent.distribution {
        Distribution::Binary(map) => {
            let platform = platform_override
                .map(str::to_string)
                .unwrap_or_else(detect_platform);
            // Look up the asset purely to report a useful error; we don't
            // download in this bead.
            let _asset = map.get(&platform).ok_or_else(|| ToolError::Sdk {
                sdk_kind: "not_found".to_string(),
                message: format!(
                    "agent `{}` has no binary asset for platform `{}` (available: {})",
                    agent.id,
                    platform,
                    map.keys().cloned().collect::<Vec<_>>().join(", ")
                ),
            })?;
            return Err(ToolError::Sdk {
                sdk_kind: "not_implemented".to_string(),
                message: format!(
                    "binary distribution install for `{}` is not yet implemented (deferred to follow-up bead)",
                    agent.id
                ),
            });
        }
        Distribution::Npx(asset) => (
            "npx",
            format!("npx -y {}@{}", asset.package, asset.version),
        ),
        Distribution::Uvx(asset) => (
            "uvx",
            format!("uvx {}=={}", asset.package, asset.version),
        ),
    };

    let entry = ProviderEntry {
        id: agent.id.clone(),
        name: agent.name.clone(),
        version: agent.version.clone(),
        distribution: distribution_kind.to_string(),
        command,
        installed_at: jiff::Timestamp::now().to_string(),
        sha256: None,
    };

    upsert_provider(&entry)?;
    serde_json::to_value(&entry).map_err(|e| ToolError::internal_message(format!("serialize provider: {e}")))
}

fn detect_platform() -> String {
    // Mirror the registry's `<os>-<arch>` triple convention.
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        other => other,
    };
    format!("{os}-{arch}")
}

// ---------------------------------------------------------------------------
// agent.uninstall
// ---------------------------------------------------------------------------

fn dispatch_uninstall(id: &str) -> Result<Value, ToolError> {
    let removed = remove_provider(id)?;
    Ok(serde_json::json!({
        "id": id,
        "removed": removed,
    }))
}

// ---------------------------------------------------------------------------
// Provider config persistence (~/.lab/acp-providers.json)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ProviderEntry {
    id: String,
    name: String,
    version: String,
    distribution: String,
    command: String,
    installed_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sha256: Option<String>,
}

fn providers_path() -> Result<std::path::PathBuf, ToolError> {
    let env_path = crate::config::dotenv_path().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "cannot determine ~/.lab path".to_string(),
    })?;
    let dir = env_path
        .parent()
        .ok_or_else(|| ToolError::internal_message("dotenv path has no parent"))?;
    Ok(dir.join("acp-providers.json"))
}

fn read_providers() -> Result<Vec<ProviderEntry>, ToolError> {
    let path = providers_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let bytes = std::fs::read(&path).map_err(|e| ToolError::internal_message(format!(
        "read {}: {e}",
        path.display()
    )))?;
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_slice(&bytes).map_err(|e| ToolError::Sdk {
        sdk_kind: "decode_error".to_string(),
        message: format!("parse {}: {e}", path.display()),
    })
}

fn write_providers(entries: &[ProviderEntry]) -> Result<(), ToolError> {
    use std::io::Write;
    let path = providers_path()?;
    let dir = path
        .parent()
        .ok_or_else(|| ToolError::internal_message("providers path has no parent"))?;
    std::fs::create_dir_all(dir).map_err(|e| ToolError::internal_message(format!(
        "create {}: {e}",
        dir.display()
    )))?;
    let mut tmp = tempfile::NamedTempFile::new_in(dir)
        .map_err(|e| ToolError::internal_message(format!("temp file: {e}")))?;
    let body = serde_json::to_vec_pretty(entries)
        .map_err(|e| ToolError::internal_message(format!("serialize providers: {e}")))?;
    tmp.write_all(&body)
        .map_err(|e| ToolError::internal_message(format!("write temp: {e}")))?;
    tmp.flush()
        .map_err(|e| ToolError::internal_message(format!("flush temp: {e}")))?;
    // Symlink-safety: refuse to overwrite a symlink at the destination.
    if let Ok(meta) = std::fs::symlink_metadata(&path)
        && meta.file_type().is_symlink()
    {
        return Err(ToolError::Sdk {
            sdk_kind: "invalid_param".to_string(),
            message: format!(
                "refusing to overwrite symlink at {} (acp-providers.json must be a regular file)",
                path.display()
            ),
        });
    }
    tmp.persist(&path).map_err(|e| ToolError::internal_message(format!(
        "persist {}: {e}",
        path.display()
    )))?;
    Ok(())
}

fn upsert_provider(entry: &ProviderEntry) -> Result<(), ToolError> {
    let mut entries = read_providers()?;
    if let Some(existing) = entries.iter_mut().find(|e| e.id == entry.id) {
        *existing = entry.clone();
    } else {
        entries.push(entry.clone());
    }
    write_providers(&entries)
}

fn remove_provider(id: &str) -> Result<bool, ToolError> {
    let mut entries = read_providers()?;
    let before = entries.len();
    entries.retain(|e| e.id != id);
    let removed = entries.len() != before;
    if removed {
        write_providers(&entries)?;
    }
    Ok(removed)
}
