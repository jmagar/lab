//! Dispatch for `agent.*` actions within the marketplace service.
//!
//! Routes ACP agent discovery and install/uninstall operations to the
//! `lab-apis::acp_registry` SDK and the local provider config at
//! `~/.lab/acp-providers.json`.
//!
//! Distribution support:
//! - `npx` and `uvx`: write a provider config entry locally; remote via fleet WS.
//! - `binary`: download archive (HTTPS only, SSRF-guarded), compute SHA-256,
//!   extract with system tar/unzip, install to `~/.lab/bin/<agent_id>/`.
//!
//! Remote install via fleet WS supports `npx` only — the device-side `DistType`
//! has no `Uvx` or `Binary` variant.

use std::path::PathBuf;

use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};
use crate::dispatch::marketplace::acp_catalog::ACP_ACTIONS;
use crate::dispatch::marketplace::acp_client;

#[cfg(feature = "acp_registry")]
use crate::dispatch::node::{NodeDispatchError, send_text_to_node};

#[cfg(feature = "acp_registry")]
use lab_apis::acp_registry::client::AcpRegistryClient;
#[cfg(feature = "acp_registry")]
use lab_apis::acp_registry::types::{Agent, BinaryAsset, Distribution};

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

    let node_ids: Vec<String> = match params.get("node_ids") {
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        Some(_) => {
            return Err(ToolError::InvalidParam {
                message: "`node_ids` must be an array of strings".to_string(),
                param: "node_ids".to_string(),
            });
        }
        None => {
            return Err(ToolError::MissingParam {
                message: "missing required parameter `node_ids`".to_string(),
                param: "node_ids".to_string(),
            });
        }
    };

    if node_ids.is_empty() {
        return Err(ToolError::InvalidParam {
            message: "`node_ids` must not be empty".to_string(),
            param: "node_ids".to_string(),
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

    let mut results = Vec::with_capacity(node_ids.len());
    for node_id in &node_ids {
        let outcome = if is_local_node(node_id) {
            install_local(&agent, &id, platform_override.as_deref()).await
        } else {
            install_remote(node_id, &agent, &id).await
        };
        match outcome {
            Ok(value) => results.push(serde_json::json!({
                "node_id": node_id,
                "ok": true,
                "result": value,
            })),
            Err(e) => results.push(serde_json::json!({
                "node_id": node_id,
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

fn is_local_node(node_id: &str) -> bool {
    if node_id.eq_ignore_ascii_case("local") {
        return true;
    }
    if let Ok(host) = std::env::var("HOSTNAME")
        && !host.is_empty()
        && node_id.eq_ignore_ascii_case(&host)
    {
        return true;
    }
    false
}

#[cfg(feature = "acp_registry")]
async fn install_local(
    agent: &Agent,
    agent_id: &str,
    platform_override: Option<&str>,
) -> Result<Value, ToolError> {
    let (distribution_kind, command, sha256) = match &agent.distribution {
        Distribution::Binary(map) => {
            let platform = platform_override
                .map(str::to_string)
                .unwrap_or_else(detect_platform);
            let asset = map.get(&platform).ok_or_else(|| ToolError::Sdk {
                sdk_kind: "not_found".to_string(),
                message: format!(
                    "agent `{}` has no binary asset for platform `{}` (available: {})",
                    agent.id,
                    platform,
                    map.keys().cloned().collect::<Vec<_>>().join(", ")
                ),
            })?;
            let (cmd_path, digest) = install_binary(agent_id, asset).await?;
            return {
                let entry = ProviderEntry {
                    id: agent_id.to_string(),
                    name: agent.name.clone(),
                    version: agent.version.clone(),
                    distribution: "binary".to_string(),
                    command: cmd_path.to_string_lossy().into_owned(),
                    installed_at: jiff::Timestamp::now().to_string(),
                    sha256: Some(digest),
                };
                upsert_provider(&entry)?;
                serde_json::to_value(&entry)
                    .map_err(|e| ToolError::internal_message(format!("serialize provider: {e}")))
            };
        }
        Distribution::Npx(asset) => (
            "npx",
            format!("npx -y {}@{}", asset.package, asset.version),
            None,
        ),
        Distribution::Uvx(asset) => (
            "uvx",
            format!("uvx {}=={}", asset.package, asset.version),
            None,
        ),
    };

    let entry = ProviderEntry {
        id: agent_id.to_string(),
        name: agent.name.clone(),
        version: agent.version.clone(),
        distribution: distribution_kind.to_string(),
        command,
        installed_at: jiff::Timestamp::now().to_string(),
        sha256,
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
// Remote fleet RPC install
// ---------------------------------------------------------------------------

/// Send an `agent.install` JSON-RPC 2.0 message to a connected remote device.
///
/// Only `npx` distribution is supported because the device-side `DistType` only
/// has an `Npx` variant. `uvx` and `binary` return a structured error.
#[cfg(feature = "acp_registry")]
async fn install_remote(
    node_id: &str,
    agent: &Agent,
    agent_id: &str,
) -> Result<Value, ToolError> {
    let (package, version) = match &agent.distribution {
        Distribution::Npx(asset) => (asset.package.clone(), asset.version.clone()),
        Distribution::Uvx(_) => {
            return Err(ToolError::Sdk {
                sdk_kind: "not_implemented".to_string(),
                message: format!(
                    "remote install of `{agent_id}` is not supported for uvx distribution \
                     (node runtime only handles npx)"
                ),
            });
        }
        Distribution::Binary(_) => {
            return Err(ToolError::Sdk {
                sdk_kind: "not_implemented".to_string(),
                message: format!(
                    "remote install of `{agent_id}` is not supported for binary distribution \
                     (node runtime only handles npx)"
                ),
            });
        }
    };

    // JSON-RPC 2.0 fire-and-forget — node processes async; we don't wait for
    // a response because `send_text_to_node` is a one-way channel. ID is 0
    // since no response correlation is needed here.
    let msg = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 0,
        "method": "agent.install",
        "params": {
            "agent_id": agent_id,
            "distribution": {
                "type": "npx",
                "package": package,
                "version": version,
            }
        }
    })
    .to_string();

    send_text_to_node(node_id, msg)
        .await
        .map_err(|e| match e {
            NodeDispatchError::NotConnected { .. } => ToolError::Sdk {
                sdk_kind: "not_found".to_string(),
                message: format!("node `{node_id}` is not connected"),
            },
            NodeDispatchError::ChannelClosed { .. } => ToolError::Sdk {
                sdk_kind: "network_error".to_string(),
                message: format!("send channel for node `{node_id}` closed (race with disconnect)"),
            },
        })?;

    Ok(serde_json::json!({
        "node_id": node_id,
        "agent_id": agent_id,
        "queued": true,
    }))
}

// ---------------------------------------------------------------------------
// Binary distribution local install
// ---------------------------------------------------------------------------

/// Download, extract, and install a binary agent to `~/.lab/bin/<agent_id>/`.
///
/// Returns `(installed_path, sha256_hex)`.
#[cfg(feature = "acp_registry")]
async fn install_binary(agent_id: &str, asset: &BinaryAsset) -> Result<(PathBuf, String), ToolError> {
    validate_archive_url(&asset.archive)?;

    let install_dir = agent_bin_dir(agent_id)?;
    std::fs::create_dir_all(&install_dir).map_err(|e| {
        ToolError::internal_message(format!("create {}: {e}", install_dir.display()))
    })?;

    // Download to a temp file next to the install dir so rename is atomic.
    let tmp_archive = tempfile::NamedTempFile::new_in(&install_dir)
        .map_err(|e| ToolError::internal_message(format!("temp archive: {e}")))?;

    let sha256 = download_archive(&asset.archive, tmp_archive.path()).await?;

    // Extract to a temp dir in the same parent so we can do an atomic move.
    let tmp_extract = tempfile::TempDir::new_in(&install_dir)
        .map_err(|e| ToolError::internal_message(format!("temp extract dir: {e}")))?;

    extract_archive(tmp_archive.path(), tmp_extract.path(), &asset.archive)?;

    // Locate the binary: `cmd` is like `"./my-agent"` or `"my-agent"`.
    let binary_name = std::path::Path::new(asset.cmd.trim_start_matches("./"))
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(asset.cmd.trim_start_matches("./"));

    let src = find_binary_in_dir(tmp_extract.path(), binary_name).ok_or_else(|| {
        ToolError::Sdk {
            sdk_kind: "not_found".to_string(),
            message: format!(
                "binary `{binary_name}` not found in extracted archive for agent `{agent_id}`"
            ),
        }
    })?;

    let dest = install_dir.join(binary_name);

    // Symlink guard before writing.
    if let Ok(meta) = std::fs::symlink_metadata(&dest)
        && meta.file_type().is_symlink()
    {
        return Err(ToolError::Sdk {
            sdk_kind: "invalid_param".to_string(),
            message: format!(
                "refusing to overwrite symlink at {} (must be a regular file)",
                dest.display()
            ),
        });
    }

    std::fs::copy(&src, &dest).map_err(|e| {
        ToolError::internal_message(format!("copy binary to {}: {e}", dest.display()))
    })?;

    #[cfg(unix)]
    {
        // lab-zxx5.18: set exact 0o755 (rwxr-xr-x). Do NOT OR with existing
        // permissions — that would preserve setuid/setgid bits (0o4000 /
        // 0o2000) if they were set on the source. An attacker-controlled
        // archive with setuid could silently install a privilege-escalation
        // binary under ~/.lab/bin/. Explicitly clear.
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755)).map_err(|e| {
            ToolError::internal_message(format!("chmod 0o755 {}: {e}", dest.display()))
        })?;
    }

    Ok((dest, sha256))
}

/// Resolve `~/.lab/bin/<agent_id>/`.
fn agent_bin_dir(agent_id: &str) -> Result<PathBuf, ToolError> {
    let env_path = crate::config::dotenv_path().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "cannot determine ~/.lab path".to_string(),
    })?;
    let lab_dir = env_path
        .parent()
        .ok_or_else(|| ToolError::internal_message("dotenv path has no parent"))?;
    Ok(lab_dir.join("bin").join(agent_id))
}

/// Validate an archive URL: require HTTPS, reject loopback/private hosts.
fn validate_archive_url(url: &str) -> Result<(), ToolError> {
    let parsed = url::Url::parse(url).map_err(|e| ToolError::Sdk {
        sdk_kind: "invalid_param".to_string(),
        message: format!("invalid archive URL `{url}`: {e}"),
    })?;
    if parsed.scheme() != "https" {
        return Err(ToolError::Sdk {
            sdk_kind: "invalid_param".to_string(),
            message: format!("archive URL must use https, got `{}`", parsed.scheme()),
        });
    }
    if let Some(host) = parsed.host_str() {
        if host == "localhost"
            || host.starts_with("127.")
            || host == "::1"
            || host.ends_with(".local")
        {
            return Err(ToolError::Sdk {
                sdk_kind: "invalid_param".to_string(),
                message: format!("archive URL host `{host}` is a local/loopback address"),
            });
        }
        if let Ok(addr) = host.parse::<std::net::IpAddr>() {
            let is_private = match addr {
                std::net::IpAddr::V4(v4) => v4.is_private() || v4.is_link_local(),
                std::net::IpAddr::V6(v6) => v6.is_unique_local() || v6.is_unicast_link_local(),
            };
            if is_private || addr.is_loopback() {
                return Err(ToolError::Sdk {
                    sdk_kind: "invalid_param".to_string(),
                    message: format!("archive URL host `{host}` is a private/loopback address"),
                });
            }
        }
    }
    Ok(())
}

/// Download `url` to `dest`, return the hex SHA-256 of the downloaded bytes.
///
/// Streaming implementation: chunks are fed to both the SHA-256 hasher and the
/// file writer concurrently so no full-archive buffer is needed in RAM.
///
/// lab-zxx5.18: download progress watchdog — if no bytes arrive for
/// `DOWNLOAD_STALL_TIMEOUT` (30s), the in-flight download is aborted,
/// the partial file is cleaned up, and an `install_timeout` error is
/// returned. This is separate from the overall reqwest timeout, which
/// caps the total duration; the watchdog catches a stalled connection
/// that's neither fast-failing nor completing.
async fn download_archive(url: &str, dest: &std::path::Path) -> Result<String, ToolError> {
    use futures::StreamExt;
    use sha2::{Digest, Sha256};
    use tokio::io::AsyncWriteExt;

    /// Abort the download if no bytes arrive within this window. Distinct
    /// from the overall `.timeout()` — catches stalled connections.
    const DOWNLOAD_STALL_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| ToolError::internal_message(format!("build http client: {e}")))?;

    let resp = client.get(url).send().await.map_err(|e| ToolError::Sdk {
        sdk_kind: "network_error".to_string(),
        message: format!("GET {url}: {e}"),
    })?;

    if !resp.status().is_success() {
        return Err(ToolError::Sdk {
            sdk_kind: "network_error".to_string(),
            message: format!("GET {url}: HTTP {}", resp.status()),
        });
    }

    let mut file = tokio::fs::File::create(dest).await.map_err(|e| {
        ToolError::internal_message(format!("create {}: {e}", dest.display()))
    })?;

    let mut hasher = Sha256::new();
    let mut stream = resp.bytes_stream();

    loop {
        // Watchdog: each chunk fetch is wrapped in a stall timeout. A download
        // that stops producing bytes within the window is treated as a fatal
        // install_timeout rather than waiting out the full request timeout.
        match tokio::time::timeout(DOWNLOAD_STALL_TIMEOUT, stream.next()).await {
            Ok(Some(chunk_result)) => {
                let chunk = chunk_result.map_err(|e| ToolError::Sdk {
                    sdk_kind: "network_error".to_string(),
                    message: format!("read body chunk from {url}: {e}"),
                })?;
                hasher.update(&chunk);
                file.write_all(&chunk).await.map_err(|e| {
                    ToolError::internal_message(format!("write chunk to {}: {e}", dest.display()))
                })?;
            }
            Ok(None) => break,
            Err(_) => {
                // Stall — clean up the partial file and surface install_timeout.
                drop(file);
                let _ = tokio::fs::remove_file(dest).await;
                return Err(ToolError::Sdk {
                    sdk_kind: "install_timeout".to_string(),
                    message: format!(
                        "download of {url} stalled for more than {:?}; aborted",
                        DOWNLOAD_STALL_TIMEOUT
                    ),
                });
            }
        }
    }

    file.flush().await.map_err(|e| {
        ToolError::internal_message(format!("flush {}: {e}", dest.display()))
    })?;

    Ok(format!("{:x}", hasher.finalize()))
}

/// Extract `archive` into `dest_dir` using system `tar` or `unzip`.
fn extract_archive(
    archive: &std::path::Path,
    dest_dir: &std::path::Path,
    url: &str,
) -> Result<(), ToolError> {
    let archive_s = archive.to_string_lossy();
    let dest_s = dest_dir.to_string_lossy();

    let status = if url.ends_with(".zip") {
        std::process::Command::new("unzip")
            .args(["-q", &archive_s, "-d", &dest_s])
            .status()
    } else {
        // .tar.gz / .tgz / .tar.xz / .tar.bz2
        let flag = if url.ends_with(".tar.xz") || url.ends_with(".txz") {
            "-xJf"
        } else {
            "-xzf"
        };
        std::process::Command::new("tar")
            .args([flag, &archive_s, "-C", &dest_s, "--no-same-owner"])
            .status()
    };

    let exit = status.map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: format!("run extraction tool: {e}"),
    })?;

    if !exit.success() {
        return Err(ToolError::Sdk {
            sdk_kind: "internal_error".to_string(),
            message: format!("extraction failed (exit {})", exit),
        });
    }
    Ok(())
}

/// Walk `dir` recursively to find the first file whose name matches `binary_name`.
fn find_binary_in_dir(dir: &std::path::Path, binary_name: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_binary_in_dir(&path, binary_name) {
                return Some(found);
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some(binary_name) {
            return Some(path);
        }
    }
    None
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

fn providers_path() -> Result<PathBuf, ToolError> {
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
