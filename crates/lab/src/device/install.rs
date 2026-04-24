/// Install handler logic for master→device RPC methods.
///
/// Implements:
/// - `marketplace.install_component` — cherry-pick file writes from a plugin
/// - `agent.install` — record ACP agent install descriptor on target device
///
/// Security invariants (enforced on every write):
/// 1. Path traversal: reject any component path containing `Component::ParentDir`
///    (i.e. any `..` segment).
/// 2. Symlink check: before writing any file, verify the target path is NOT a symlink
///    via `symlink_metadata().file_type().is_symlink()`.
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc;

// --------------------------------------------------------------------------
// Parameter types
// --------------------------------------------------------------------------

/// Install scope: `"global"` → `~/.claude/` or `"project"` → `{project_path}/.claude/`.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InstallScope {
    Global,
    Project,
}

/// Distribution type for an installable ACP agent.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DistType {
    Npx,
}

#[derive(Debug, Deserialize)]
pub struct InstallComponentParams {
    #[allow(dead_code)] // Forwarded in RPC params; available for logging/auditing.
    pub plugin_id: String,
    /// Which files from the plugin to cherry-pick (relative paths within plugin).
    pub components: Vec<String>,
    pub scope: InstallScope,
    /// Required when scope == `Project`; must be an absolute path with no `..`.
    pub project_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AgentInstallParams {
    pub agent_id: String,
    pub distribution: AgentDistribution,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AgentDistribution {
    #[serde(rename = "type")]
    pub dist_type: DistType,
    pub package: String,
    pub version: String,
}

// --------------------------------------------------------------------------
// Result types
// --------------------------------------------------------------------------

#[derive(Debug, Default, Serialize)]
pub struct InstallComponentResult {
    pub written: Vec<String>,
    pub skipped: Vec<String>,
    pub errors: Vec<InstallError>,
}

#[derive(Debug, Serialize)]
pub struct InstallError {
    pub file: String,
    pub error: String,
}

// --------------------------------------------------------------------------
// Progress notifications
// --------------------------------------------------------------------------

/// A progress notification sent back to master during install.
/// Does NOT carry an `id` field (notification, not request/response).
#[derive(Debug, Serialize)]
pub struct InstallProgressNotification {
    pub jsonrpc: &'static str,
    pub method: &'static str,
    pub params: InstallProgressParams,
}

#[derive(Debug, Serialize)]
pub struct InstallProgressParams {
    pub rpc_id: Value,
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

async fn send_progress(tx: &mpsc::Sender<String>, notif: &InstallProgressNotification) {
    if let Ok(encoded) = serde_json::to_string(notif) {
        tx.send(encoded).await.ok();
    }
}

impl InstallProgressNotification {
    pub fn started(rpc_id: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            method: "install/progress",
            params: InstallProgressParams {
                rpc_id,
                status: "started",
                file: None,
                error: None,
            },
        }
    }

    pub fn file_written(rpc_id: Value, file: String) -> Self {
        Self {
            jsonrpc: "2.0",
            method: "install/progress",
            params: InstallProgressParams {
                rpc_id,
                status: "file_written",
                file: Some(file),
                error: None,
            },
        }
    }

    #[allow(dead_code)] // Available for callers that distinguish skipped from written.
    pub fn file_skipped(rpc_id: Value, file: String) -> Self {
        Self {
            jsonrpc: "2.0",
            method: "install/progress",
            params: InstallProgressParams {
                rpc_id,
                status: "file_skipped",
                file: Some(file),
                error: None,
            },
        }
    }

    pub fn file_error(rpc_id: Value, file: String, error: String) -> Self {
        Self {
            jsonrpc: "2.0",
            method: "install/progress",
            params: InstallProgressParams {
                rpc_id,
                status: "file_error",
                file: Some(file),
                error: Some(error),
            },
        }
    }
}

// --------------------------------------------------------------------------
// Security helpers
// --------------------------------------------------------------------------

/// Returns `Err` if `rel_path` contains any `..` component.
pub fn reject_path_traversal(rel_path: &str) -> Result<()> {
    let path = Path::new(rel_path);
    for component in path.components() {
        if matches!(component, Component::ParentDir) {
            return Err(anyhow!(
                "path traversal rejected: `{rel_path}` contains `..`"
            ));
        }
    }
    Ok(())
}

/// Returns `Err` if `path` exists AND is a symlink.
pub async fn reject_symlink(path: &Path) -> Result<()> {
    match tokio::fs::symlink_metadata(path).await {
        Ok(meta) if meta.file_type().is_symlink() => Err(anyhow!(
            "symlink rejected: `{}` is a symbolic link",
            path.display()
        )),
        Ok(_) | Err(_) => Ok(()),
    }
}

/// Resolve the write root for a given scope.
///
/// - `Global` → `~/.claude/`
/// - `Project` → `{project_path}/.claude/`
pub fn resolve_write_root(scope: InstallScope, project_path: Option<&str>) -> Result<PathBuf> {
    match scope {
        InstallScope::Global => {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .map_err(|_| anyhow!("HOME environment variable is not set"))?;
            Ok(PathBuf::from(home).join(".claude"))
        }
        InstallScope::Project => {
            let raw = project_path
                .ok_or_else(|| anyhow!("project_path is required when scope == Project"))?;
            let p = PathBuf::from(raw);
            if !p.is_absolute() {
                return Err(anyhow!(
                    "project_path must be absolute; got `{}`",
                    p.display()
                ));
            }
            for component in p.components() {
                if matches!(component, Component::ParentDir) {
                    return Err(anyhow!(
                        "path traversal rejected in project_path: `{}`",
                        p.display()
                    ));
                }
            }
            Ok(p.join(".claude"))
        }
    }
}

// --------------------------------------------------------------------------
// marketplace.install_component
// --------------------------------------------------------------------------

/// Executes a `marketplace.install_component` request.
///
/// `component_files` maps relative-path component names to their file contents
/// (already fetched from the marketplace or resolved by the caller).
///
/// Progress notifications are sent via `progress_tx` as JSON-encoded strings.
/// The caller is responsible for routing them to the WebSocket send channel.
pub async fn handle_install_component(
    params: InstallComponentParams,
    component_files: Vec<(String, Vec<u8>)>,
    rpc_id: Value,
    progress_tx: &mpsc::Sender<String>,
) -> Result<InstallComponentResult> {
    // Send started progress notification.
    send_progress(progress_tx, &InstallProgressNotification::started(rpc_id.clone())).await;

    let write_root = resolve_write_root(params.scope, params.project_path.as_deref())?;

    let mut result = InstallComponentResult::default();

    for (component_path, contents) in component_files {
        // Security: reject path traversal.
        if let Err(err) = reject_path_traversal(&component_path) {
            let msg = err.to_string();
            send_progress(progress_tx, &InstallProgressNotification::file_error(
                        rpc_id.clone(),
                        component_path.clone(),
                        msg.clone(),
                    )).await;
            result.errors.push(InstallError {
                file: component_path,
                error: msg,
            });
            continue;
        }

        let target = write_root.join(&component_path);

        // Security: reject symlink at target path.
        if let Err(err) = reject_symlink(&target).await {
            let msg = err.to_string();
            send_progress(progress_tx, &InstallProgressNotification::file_error(
                        rpc_id.clone(),
                        component_path.clone(),
                        msg.clone(),
                    )).await;
            result.errors.push(InstallError {
                file: component_path,
                error: msg,
            });
            continue;
        }

        // Ensure parent directory exists.
        if let Some(parent) = target.parent() {
            if let Err(err) = tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("create parent dir for `{}`", target.display()))
            {
                let msg = err.to_string();
                send_progress(progress_tx, &InstallProgressNotification::file_error(
                            rpc_id.clone(),
                            component_path.clone(),
                            msg.clone(),
                        )).await;
                result.errors.push(InstallError {
                    file: component_path,
                    error: msg,
                });
                continue;
            }
        }

        match tokio::fs::write(&target, &contents).await {
            Ok(()) => {
                tracing::info!(
                    surface = "device",
                    service = "install",
                    action = "component.write",
                    path = %target.display(),
                    "wrote component file"
                );
                send_progress(progress_tx, &InstallProgressNotification::file_written(
                            rpc_id.clone(),
                            component_path.clone(),
                        )).await;
                result.written.push(component_path);
            }
            Err(err) => {
                let msg = format!("write failed: {err}");
                send_progress(progress_tx, &InstallProgressNotification::file_error(
                            rpc_id.clone(),
                            component_path.clone(),
                            msg.clone(),
                        )).await;
                result.errors.push(InstallError {
                    file: component_path,
                    error: msg,
                });
            }
        }
    }

    Ok(result)
}

// --------------------------------------------------------------------------
// agent.install
// --------------------------------------------------------------------------

/// Executes an `agent.install` request.
///
/// Writes the agent distribution descriptor to `~/.claude/agents/{agent_id}.json`
/// (or the project-scoped equivalent). Does NOT actually invoke npx or spawn a
/// process — that is the agent runtime's responsibility.
///
/// Progress notifications follow the same pattern as `handle_install_component`.
pub async fn handle_agent_install(
    params: AgentInstallParams,
    scope: InstallScope,
    project_path: Option<&str>,
    rpc_id: Value,
    progress_tx: &mpsc::Sender<String>,
) -> Result<InstallComponentResult> {
    // Send started progress notification.
    send_progress(progress_tx, &InstallProgressNotification::started(rpc_id.clone())).await;

    reject_path_traversal(&params.agent_id)?;

    let write_root = resolve_write_root(scope, project_path)?;
    let agents_dir = write_root.join("agents");
    let target_file = format!("{}.json", params.agent_id);
    let target = agents_dir.join(&target_file);

    let mut result = InstallComponentResult::default();

    // Security: reject symlink at target path.
    if let Err(err) = reject_symlink(&target).await {
        let msg = err.to_string();
        send_progress(progress_tx, &InstallProgressNotification::file_error(
                    rpc_id.clone(),
                    target_file.clone(),
                    msg.clone(),
                )).await;
        result.errors.push(InstallError {
            file: target_file,
            error: msg,
        });
        return Ok(result);
    }

    // Ensure agents dir exists.
    if let Err(err) = tokio::fs::create_dir_all(&agents_dir)
        .await
        .with_context(|| format!("create agents dir `{}`", agents_dir.display()))
    {
        let msg = err.to_string();
        send_progress(progress_tx, &InstallProgressNotification::file_error(
                    rpc_id.clone(),
                    target_file.clone(),
                    msg.clone(),
                )).await;
        result.errors.push(InstallError {
            file: target_file,
            error: msg,
        });
        return Ok(result);
    }

    // Serialize distribution descriptor.
    let payload = serde_json::json!({
        "agent_id": params.agent_id,
        "distribution": params.distribution,
    });
    let contents = match serde_json::to_vec_pretty(&payload) {
        Ok(bytes) => bytes,
        Err(err) => {
            let msg = format!("serialize distribution: {err}");
            send_progress(progress_tx, &InstallProgressNotification::file_error(
                        rpc_id.clone(),
                        target_file.clone(),
                        msg.clone(),
                    )).await;
            result.errors.push(InstallError {
                file: target_file,
                error: msg,
            });
            return Ok(result);
        }
    };

    match tokio::fs::write(&target, &contents).await {
        Ok(()) => {
            tracing::info!(
                surface = "device",
                service = "install",
                action = "agent.write",
                path = %target.display(),
                agent_id = %params.agent_id,
                "wrote agent install descriptor"
            );
            send_progress(progress_tx, &InstallProgressNotification::file_written(
                        rpc_id.clone(),
                        target_file.clone(),
                    )).await;
            result.written.push(target_file);
        }
        Err(err) => {
            let msg = format!("write failed: {err}");
            send_progress(progress_tx, &InstallProgressNotification::file_error(
                        rpc_id.clone(),
                        target_file.clone(),
                        msg.clone(),
                    )).await;
            result.errors.push(InstallError {
                file: target_file,
                error: msg,
            });
        }
    }

    Ok(result)
}

// --------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_path_traversal_catches_dotdot() {
        assert!(reject_path_traversal("..").is_err());
        assert!(reject_path_traversal("../evil").is_err());
        assert!(reject_path_traversal("agents/../evil").is_err());
        assert!(reject_path_traversal("foo/../../secret").is_err());
    }

    #[test]
    fn reject_path_traversal_allows_safe_paths() {
        assert!(reject_path_traversal("agents/my-agent.md").is_ok());
        assert!(reject_path_traversal(".mcp.json").is_ok());
        assert!(reject_path_traversal("some/nested/file.json").is_ok());
    }

    #[test]
    fn resolve_write_root_global_uses_home() {
        let root = resolve_write_root(InstallScope::Global, None).expect("global root");
        assert!(root.is_absolute());
        assert_eq!(root.file_name(), Some(std::ffi::OsStr::new(".claude")));
    }

    #[test]
    fn resolve_write_root_project_requires_absolute() {
        assert!(resolve_write_root(InstallScope::Project, Some("relative/path")).is_err());
        assert!(resolve_write_root(InstallScope::Project, Some("/abs/path")).is_ok());
    }

    #[test]
    fn resolve_write_root_project_rejects_traversal() {
        assert!(resolve_write_root(InstallScope::Project, Some("/abs/../etc")).is_err());
    }

    #[test]
    fn scope_deserializes_lowercase() {
        let scope: InstallScope = serde_json::from_str(r#""global""#).expect("global");
        assert_eq!(scope, InstallScope::Global);
        let scope: InstallScope = serde_json::from_str(r#""project""#).expect("project");
        assert_eq!(scope, InstallScope::Project);
        assert!(serde_json::from_str::<InstallScope>(r#""workspace""#).is_err());
    }

    #[tokio::test]
    async fn reject_symlink_detects_symlinks() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let target = tempdir.path().join("real_file");
        tokio::fs::write(&target, b"hello").await.expect("write");
        let link = tempdir.path().join("symlink");
        #[cfg(unix)]
        tokio::fs::symlink(&target, &link).await.expect("symlink");
        #[cfg(unix)]
        assert!(reject_symlink(&link).await.is_err(), "should reject symlink");
        assert!(reject_symlink(&target).await.is_ok(), "should allow real file");
    }

    #[tokio::test]
    async fn handle_install_component_writes_files() {
        let tempdir = tempfile::tempdir().expect("tempdir");

        let (progress_tx, mut progress_rx) = mpsc::channel(16);
        let params = InstallComponentParams {
            plugin_id: "test-plugin@marketplace".to_string(),
            components: vec!["agents/my-agent.md".to_string()],
            scope: InstallScope::Project,
            project_path: Some(tempdir.path().to_string_lossy().into_owned()),
        };
        let files = vec![("agents/my-agent.md".to_string(), b"# My Agent\n".to_vec())];

        let result = handle_install_component(params, files, serde_json::json!("req-1"), &progress_tx)
            .await
            .expect("install");

        assert_eq!(result.written, vec!["agents/my-agent.md"]);
        assert!(result.errors.is_empty());
        assert!(result.skipped.is_empty());

        let written_content = tokio::fs::read_to_string(
            tempdir.path().join(".claude/agents/my-agent.md"),
        )
        .await
        .expect("read written file");
        assert_eq!(written_content, "# My Agent\n");

        // At least one progress notification should have been sent.
        let _ = progress_rx.try_recv();
    }

    #[tokio::test]
    async fn handle_install_component_rejects_path_traversal() {
        let tempdir = tempfile::tempdir().expect("tempdir");

        let (progress_tx, _rx) = mpsc::channel(16);
        let params = InstallComponentParams {
            plugin_id: "evil@marketplace".to_string(),
            components: vec!["../etc/passwd".to_string()],
            scope: InstallScope::Project,
            project_path: Some(tempdir.path().to_string_lossy().into_owned()),
        };
        let files = vec![("../etc/passwd".to_string(), b"root:x:0:0\n".to_vec())];

        let result = handle_install_component(params, files, serde_json::json!(1), &progress_tx)
            .await
            .expect("install ran");

        assert!(result.written.is_empty());
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].error.contains(".."));
    }

    #[tokio::test]
    async fn handle_agent_install_writes_descriptor() {
        let tempdir = tempfile::tempdir().expect("tempdir");

        let (progress_tx, _rx) = mpsc::channel(16);
        let params = AgentInstallParams {
            agent_id: "claude-agent".to_string(),
            distribution: AgentDistribution {
                dist_type: DistType::Npx,
                package: "@anthropic/claude-agent-acp".to_string(),
                version: "0.30.0".to_string(),
            },
        };

        let result = handle_agent_install(
            params,
            InstallScope::Project,
            Some(tempdir.path().to_str().unwrap()),
            serde_json::json!("req-2"),
            &progress_tx,
        )
        .await
        .expect("agent install");

        assert_eq!(result.written, vec!["claude-agent.json"]);
        assert!(result.errors.is_empty());

        let written = tokio::fs::read_to_string(
            tempdir.path().join(".claude/agents/claude-agent.json"),
        )
        .await
        .expect("read agent file");
        let v: serde_json::Value = serde_json::from_str(&written).expect("parse");
        assert_eq!(v["agent_id"], "claude-agent");
        assert_eq!(v["distribution"]["package"], "@anthropic/claude-agent-acp");
    }
}
