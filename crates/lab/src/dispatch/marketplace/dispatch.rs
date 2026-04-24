use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use lab_apis::marketplace::{Artifact, ArtifactLang, Marketplace, Plugin, PluginSource};
use serde::Serialize;
use serde_json::{Map, Value};
use tempfile::NamedTempFile;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{
    action_schema, help_payload, optional_str, require_str, to_json,
};
use crate::dispatch::marketplace::acp_dispatch;
use crate::dispatch::marketplace::catalog::ACTIONS;
use crate::dispatch::marketplace::client;
use crate::dispatch::marketplace::params::{self, parse_plugin_id};

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    dispatch_with_port(action, params, &client::NoopDeviceRpcPort).await
}

pub(crate) async fn dispatch_with_port(
    action: &str,
    params: Value,
    port: &dyn client::DeviceRpcPort,
) -> Result<Value, ToolError> {
    if action.starts_with("agent.") {
        return acp_dispatch::dispatch_acp(action, params).await;
    }
    match action {
        "help" => Ok(help_payload("marketplace", ACTIONS)),
        "schema" => {
            let a = require_str(&params, "action")?;
            action_schema(ACTIONS, a)
        }
        "sources.list" => sources_list().await,
        "sources.add" => {
            let repo = optional_str(&params, "repo")?.map(ToString::to_string);
            let url = optional_str(&params, "url")?.map(ToString::to_string);
            let auto_update = params.get("autoUpdate").and_then(Value::as_bool);
            sources_add(repo, url, auto_update).await
        }
        "plugins.list" => {
            let filter = optional_str(&params, "marketplace")?.map(ToString::to_string);
            plugins_list(filter).await
        }
        "plugin.get" => {
            let id = require_str(&params, "id")?.to_string();
            plugin_get(&id).await
        }
        "plugin.artifacts" => {
            let id = require_str(&params, "id")?.to_string();
            plugin_artifacts(&id).await
        }
        "plugin.workspace" => {
            let id = require_str(&params, "id")?.to_string();
            plugin_workspace(&id).await
        }
        "plugin.save" => {
            let id = require_str(&params, "id")?.to_string();
            let path = require_str(&params, "path")?.to_string();
            let content = require_str(&params, "content")?.to_string();
            plugin_save(&id, &path, &content).await
        }
        "plugin.deploy" => {
            let id = require_str(&params, "id")?.to_string();
            plugin_deploy(&id).await
        }
        "plugin.deploy.preview" => {
            let id = require_str(&params, "id")?.to_string();
            plugin_deploy_preview(&id).await
        }
        "plugin.install" => {
            let id = require_str(&params, "id")?.to_string();
            plugin_shell("install", &id).await
        }
        "plugin.uninstall" => {
            let id = require_str(&params, "id")?.to_string();
            plugin_shell("uninstall", &id).await
        }
        "plugin.cherry_pick" => {
            let cp = params::parse_cherry_pick_params(&params)?;
            plugin_cherry_pick(cp, port).await
        }
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action `marketplace.{unknown}`"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

fn read_json(path: &Path) -> Result<Value, ToolError> {
    let bytes = std::fs::read(path).map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: format!("read {}: {e}", path.display()),
    })?;
    serde_json::from_slice(&bytes).map_err(|e| ToolError::Sdk {
        sdk_kind: "decode_error".into(),
        message: format!("parse {}: {e}", path.display()),
    })
}

// ---------- internal raw types (from ~/.claude/plugins/*.json) ----------

fn load_known_marketplaces() -> Result<Vec<Marketplace>, ToolError> {
    let path = client::plugins_root()?.join("known_marketplaces.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let v = read_json(&path)?;
    let Some(obj) = v.as_object() else {
        return Ok(Vec::new());
    };
    let mut out = Vec::with_capacity(obj.len());
    for (id, entry) in obj {
        let src = entry.get("source").cloned().unwrap_or(Value::Null);
        let (source, url, repo, path_val, gh_user) = match src {
            Value::Object(m) => parse_source(&m),
            _ => (PluginSource::Local, None, None, None, String::new()),
        };
        let auto_update = entry
            .get("autoUpdate")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let last_updated = entry
            .get("lastUpdated")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let install_loc = entry
            .get("installLocation")
            .and_then(Value::as_str)
            .map(PathBuf::from);
        let manifest = install_loc.as_deref().and_then(read_marketplace_manifest);
        let plugin_count = manifest.as_ref().map(|m| m.plugins.len()).unwrap_or(0);
        let display_name = manifest
            .as_ref()
            .and_then(|m| m.display_name.clone())
            .unwrap_or_else(|| id.clone());
        let owner = manifest
            .as_ref()
            .and_then(|m| m.owner_name.clone())
            .unwrap_or_else(|| gh_user.clone());
        let desc = manifest
            .as_ref()
            .and_then(|m| m.description.clone())
            .unwrap_or_default();
        out.push(Marketplace {
            id: id.clone(),
            name: display_name,
            owner,
            gh_user,
            repo,
            source,
            url,
            path: path_val,
            desc,
            auto_update,
            total_plugins: plugin_count as u32,
            last_updated,
            runtime: None,
        });
    }
    Ok(out)
}

fn parse_source(
    m: &Map<String, Value>,
) -> (
    PluginSource,
    Option<String>,
    Option<String>,
    Option<String>,
    String,
) {
    let kind = m.get("source").and_then(Value::as_str).unwrap_or("local");
    let url = m.get("url").and_then(Value::as_str).map(ToString::to_string);
    let repo = m.get("repo").and_then(Value::as_str).map(ToString::to_string);
    let path = m
        .get("path")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let gh_user = repo
        .as_deref()
        .and_then(|r| r.split('/').next())
        .unwrap_or("")
        .to_string();
    let source = match kind {
        "github" => PluginSource::Github,
        "git" => PluginSource::Git,
        _ => PluginSource::Local,
    };
    (source, url, repo, path, gh_user)
}

/// Parsed representation of a marketplace.json file.
struct MarketplaceManifest {
    display_name: Option<String>,
    owner_name: Option<String>,
    description: Option<String>,
    plugins: Vec<Value>,
}

fn read_marketplace_manifest(install_loc: &Path) -> Option<MarketplaceManifest> {
    let candidates = [
        install_loc.join(".claude-plugin").join("marketplace.json"),
        install_loc.join("marketplace.json"),
    ];
    for p in &candidates {
        if p.exists() {
            if let Ok(v) = read_json(p) {
                let plugins = v
                    .get("plugins")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let display_name = v
                    .get("name")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                let owner_name = v
                    .get("owner")
                    .and_then(|o| o.get("name"))
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                let description = v
                    .get("metadata")
                    .and_then(|m| m.get("description"))
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                return Some(MarketplaceManifest {
                    display_name,
                    owner_name,
                    description,
                    plugins,
                });
            }
        }
    }
    None
}

/// Map of installed plugin id (`name@marketplace`) → installPath + timestamps.
struct InstalledRecord {
    install_path: PathBuf,
    installed_at: String,
    last_updated: String,
}

#[derive(Debug, Clone, Serialize)]
struct PluginWorkspace {
    #[serde(rename = "pluginId")]
    plugin_id: String,
    files: Vec<Artifact>,
    #[serde(rename = "hasDirtyFiles")]
    has_dirty_files: bool,
    #[serde(rename = "deployTarget", skip_serializing_if = "Option::is_none")]
    deploy_target: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SaveResult {
    #[serde(rename = "savedAt")]
    saved_at: String,
}

fn load_installed() -> Result<HashMap<String, InstalledRecord>, ToolError> {
    let path = client::plugins_root()?.join("installed_plugins.json");
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let v = read_json(&path)?;
    let Some(obj) = v.get("plugins").and_then(Value::as_object) else {
        return Ok(HashMap::new());
    };
    let mut out = HashMap::new();
    for (id, list) in obj {
        if let Some(first) = list.as_array().and_then(|a| a.first()) {
            out.insert(
                id.clone(),
                InstalledRecord {
                    install_path: first
                        .get("installPath")
                        .and_then(Value::as_str)
                        .map(PathBuf::from)
                        .unwrap_or_default(),
                    installed_at: first
                        .get("installedAt")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                    last_updated: first
                        .get("lastUpdated")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                },
            );
        }
    }
    Ok(out)
}

fn build_plugin(
    mkt_id: &str,
    plugin_json: &Value,
    installed: &HashMap<String, InstalledRecord>,
) -> Option<Plugin> {
    let name = plugin_json.get("name").and_then(Value::as_str)?.to_string();
    let id = format!("{name}@{mkt_id}");
    let ver = plugin_json
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let desc = plugin_json
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let mut tags: Vec<String> = Vec::new();
    for key in ["tags", "keywords"] {
        if let Some(arr) = plugin_json.get(key).and_then(Value::as_array) {
            for v in arr {
                if let Some(s) = v.as_str() {
                    let s = s.to_string();
                    if !tags.contains(&s) {
                        tags.push(s);
                    }
                }
            }
        }
    }
    if let Some(cat) = plugin_json.get("category").and_then(Value::as_str) {
        let cat = cat.to_string();
        if !tags.contains(&cat) {
            tags.insert(0, cat);
        }
    }
    let rec = installed.get(&id);
    Some(Plugin {
        id,
        name,
        mkt: mkt_id.to_string(),
        ver,
        desc,
        tags,
        installed: rec.is_some(),
        has_update: None,
        installed_at: rec.map(|r| r.installed_at.clone()),
        updated_at: rec.map(|r| r.last_updated.clone()),
        runtime: None,
        enabled: None,
        marketplace_id: None,
        version: None,
        description: None,
        manifest: None,
        components: None,
        install_state: None,
        source_path: None,
        cache_path: None,
    })
}

// ---------- action handlers ----------

async fn sources_list() -> Result<Value, ToolError> {
    let markets = tokio::task::spawn_blocking(load_known_marketplaces)
        .await
        .map_err(join_err)??;
    to_json(markets)
}

async fn plugins_list(filter: Option<String>) -> Result<Value, ToolError> {
    let plugins = tokio::task::spawn_blocking(move || -> Result<Vec<Plugin>, ToolError> {
        let markets = load_known_marketplaces()?;
        let installed = load_installed()?;
        let mut out = Vec::new();
        for m in markets {
            if let Some(ref f) = filter {
                if &m.id != f {
                    continue;
                }
            }
            let install_loc = client::plugins_root()?
                .join("marketplaces")
                .join(&m.id);
            let Some(manifest) = read_marketplace_manifest(&install_loc) else {
                continue;
            };
            for pj in &manifest.plugins {
                if let Some(p) = build_plugin(&m.id, pj, &installed) {
                    out.push(p);
                }
            }
        }
        Ok(out)
    })
    .await
    .map_err(join_err)??;
    to_json(plugins)
}

async fn plugin_get(id: &str) -> Result<Value, ToolError> {
    let (name, mkt) = parse_plugin_id(id)?;
    let (name, mkt) = (name.to_string(), mkt.to_string());
    let found = tokio::task::spawn_blocking(move || -> Result<Option<Plugin>, ToolError> {
        let installed = load_installed()?;
        let install_loc = client::plugins_root()?.join("marketplaces").join(&mkt);
        let Some(manifest) = read_marketplace_manifest(&install_loc) else {
            return Ok(None);
        };
        for pj in &manifest.plugins {
            if pj.get("name").and_then(Value::as_str) == Some(name.as_str()) {
                return Ok(build_plugin(&mkt, pj, &installed));
            }
        }
        Ok(None)
    })
    .await
    .map_err(join_err)??;

    match found {
        Some(p) => to_json(p),
        None => Err(ToolError::Sdk {
            sdk_kind: "not_found".into(),
            message: format!("plugin `{id}` not found"),
        }),
    }
}

async fn plugin_artifacts(id: &str) -> Result<Value, ToolError> {
    parse_plugin_id(id)?;
    let id_owned = id.to_string();
    let artifacts = tokio::task::spawn_blocking(move || -> Result<Vec<Artifact>, ToolError> {
        let source = source_path_for_plugin(&id_owned)?;
        walk_artifacts(&source, &source)
    })
    .await
    .map_err(join_err)??;
    to_json(artifacts)
}

async fn plugin_workspace(id: &str) -> Result<Value, ToolError> {
    parse_plugin_id(id)?;
    let id_owned = id.to_string();
    let workspace = tokio::task::spawn_blocking(move || -> Result<PluginWorkspace, ToolError> {
        let dir = ensure_workspace_for_plugin(&id_owned)?;
        let target = installed_target_for_plugin(&id_owned)?
            .map(|path| path.to_string_lossy().into_owned());
        let files = walk_artifacts(&dir, &dir)?;
        Ok(PluginWorkspace {
            plugin_id: id_owned,
            files,
            has_dirty_files: false,
            deploy_target: target,
        })
    })
    .await
    .map_err(join_err)??;
    to_json(workspace)
}

async fn plugin_save(id: &str, rel_path: &str, content: &str) -> Result<Value, ToolError> {
    parse_plugin_id(id)?;
    let id_owned = id.to_string();
    let rel_owned = rel_path.to_string();
    let content_owned = content.to_string();
    let result = tokio::task::spawn_blocking(move || -> Result<SaveResult, ToolError> {
        let workspace = ensure_workspace_for_plugin(&id_owned)?;
        let target = resolve_relative_path(&workspace, &rel_owned)?;
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(io_internal)?;
        }
        let temp_dir = target.parent().unwrap_or(&workspace);
        let mut temp = NamedTempFile::new_in(temp_dir).map_err(io_internal)?;
        temp.write_all(content_owned.as_bytes()).map_err(io_internal)?;
        temp.flush().map_err(io_internal)?;
        temp.persist(&target).map_err(|err| io_internal(err.error))?;
        Ok(SaveResult {
            saved_at: jiff::Timestamp::now().to_string(),
        })
    })
    .await
    .map_err(join_err)??;
    to_json(result)
}

async fn plugin_deploy(id: &str) -> Result<Value, ToolError> {
    parse_plugin_id(id)?;
    let id_owned = id.to_string();
    let result = tokio::task::spawn_blocking(move || -> Result<client::DeployResult, ToolError> {
        let workspace = ensure_workspace_for_plugin(&id_owned)?;
        let target = installed_target_for_plugin(&id_owned)?.ok_or_else(|| ToolError::Sdk {
            sdk_kind: "not_found".into(),
            message: format!("plugin `{id_owned}` must be installed before it can be deployed"),
        })?;
        client::sync_workspace_to_target(&workspace, &target)
    })
    .await
    .map_err(join_err)??;
    to_json(result)
}

async fn plugin_deploy_preview(id: &str) -> Result<Value, ToolError> {
    parse_plugin_id(id)?;
    let id_owned = id.to_string();
    let result =
        tokio::task::spawn_blocking(move || -> Result<client::DeployPreviewResult, ToolError> {
        let workspace = ensure_workspace_for_plugin(&id_owned)?;
        let target = installed_target_for_plugin(&id_owned)?.ok_or_else(|| ToolError::Sdk {
            sdk_kind: "not_found".into(),
            message: format!("plugin `{id_owned}` must be installed before deployment can be previewed"),
        })?;
        client::preview_workspace_sync(&workspace, &target)
    })
    .await
    .map_err(join_err)??;
    to_json(result)
}

const MAX_ARTIFACTS: usize = 200;
const MAX_ARTIFACT_BYTES: u64 = 256 * 1024;

pub(crate) fn walk_artifacts(root: &Path, dir: &Path) -> Result<Vec<Artifact>, ToolError> {
    let mut out = Vec::new();
    walk_artifacts_into(root, dir, &mut out)?;
    Ok(out)
}

fn walk_artifacts_into(root: &Path, dir: &Path, out: &mut Vec<Artifact>) -> Result<(), ToolError> {
    if out.len() >= MAX_ARTIFACTS {
        return Ok(());
    }

    let rd = std::fs::read_dir(dir).map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: format!("read_dir {}: {e}", dir.display()),
    })?;
    for entry in rd.flatten() {
        if out.len() >= MAX_ARTIFACTS {
            break;
        }
        let p = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == ".git" || name == "node_modules" || name == "target" {
            continue;
        }
        let ft = match entry.file_type() {
            Ok(ft) => ft,
            Err(error) => {
                tracing::warn!(
                    service = "marketplace",
                    event = "artifact.read.file_type_failed",
                    path = %p.display(),
                    error = %error,
                    "could not determine file type; skipping entry"
                );
                continue;
            }
        };
        if ft.is_symlink() {
            tracing::warn!(
                service = "marketplace",
                event = "artifact.read.skipped",
                path = %p.display(),
                "skipping symlink"
            );
            continue;
        }
        if ft.is_dir() {
            walk_artifacts_into(root, &p, out)?;
            continue;
        }
        if entry.metadata().ok().is_some_and(|m| m.len() > MAX_ARTIFACT_BYTES) {
            continue;
        }
        let rel = p
            .strip_prefix(root)
            .unwrap_or(&p)
            .to_string_lossy()
            .into_owned();
        let lang = detect_lang(&p);
        let bytes = match std::fs::read(&p) {
            Ok(bytes) => bytes,
            Err(error) => {
                tracing::warn!(
                    service = "marketplace",
                    event = "artifact.read.skipped",
                    path = %p.display(),
                    error = %error,
                    "skipping unreadable artifact"
                );
                continue;
            }
        };
        let content = match String::from_utf8(bytes) {
            Ok(content) => content,
            Err(error) => {
                tracing::warn!(
                    service = "marketplace",
                    event = "artifact.read.skipped",
                    path = %p.display(),
                    error = %error,
                    "skipping non-utf8 artifact"
                );
                continue;
            }
        };
        out.push(Artifact { path: rel, lang, content });
    }
    Ok(())
}

fn workspace_root() -> Result<PathBuf, ToolError> {
    Ok(client::plugins_root()?.join("workspaces"))
}

fn workspace_dir_for_plugin(id: &str) -> Result<PathBuf, ToolError> {
    Ok(workspace_root()?.join(sanitize_plugin_id(id)))
}

fn sanitize_plugin_id(id: &str) -> String {
    id.chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' => '_',
            other => other,
        })
        .collect()
}

fn source_path_for_plugin(id: &str) -> Result<PathBuf, ToolError> {
    let (name, marketplace) = parse_plugin_id(id)?;
    let root = client::plugins_root()?;
    let candidate = root.join("marketplaces").join(marketplace).join(name);
    if candidate.exists() {
        // Belt-and-suspenders: canonicalize to detect any residual traversal.
        let canonical = std::fs::canonicalize(&candidate).map_err(io_internal)?;
        let canonical_root = std::fs::canonicalize(&root).map_err(io_internal)?;
        if !canonical.starts_with(&canonical_root) {
            return Err(ToolError::InvalidParam {
                param: "id".into(),
                message: format!("plugin id `{id}` resolves outside the plugins directory"),
            });
        }
        return Ok(canonical);
    }
    let installed = load_installed()?;
    if let Some(record) = installed.get(id) {
        return Ok(record.install_path.clone());
    }
    Err(ToolError::Sdk {
        sdk_kind: "not_found".into(),
        message: format!("no local plugin source found for `{id}`"),
    })
}

fn ensure_workspace_for_plugin(id: &str) -> Result<PathBuf, ToolError> {
    let workspace = workspace_dir_for_plugin(id)?;
    if workspace.exists() {
        return Ok(workspace);
    }
    let source = source_path_for_plugin(id)?;
    std::fs::create_dir_all(&workspace).map_err(io_internal)?;
    copy_tree(&source, &workspace)?;
    Ok(workspace)
}

fn installed_target_for_plugin(id: &str) -> Result<Option<PathBuf>, ToolError> {
    let Some(record) = load_installed()?.remove(id) else {
        return Ok(None);
    };
    let path = record.install_path;
    if path.as_os_str().is_empty() {
        return Err(ToolError::InvalidParam {
            param: "id".into(),
            message: format!("install path for `{id}` is empty"),
        });
    }
    let root = client::plugins_root()?;
    let canonical_root = std::fs::canonicalize(&root).map_err(io_internal)?;

    // Resolve to an absolute, root-anchored path without following `..`.
    let resolved = if path.is_absolute() {
        // Reject any `..` or prefix/root-other components; only ordinary path segments allowed
        // after the root so the prefix check cannot be bypassed textually.
        for component in path.components() {
            match component {
                std::path::Component::Normal(_)
                | std::path::Component::RootDir
                | std::path::Component::Prefix(_) => {}
                std::path::Component::ParentDir | std::path::Component::CurDir => {
                    return Err(ToolError::InvalidParam {
                        param: "id".into(),
                        message: format!(
                            "install path for `{id}` contains invalid path components"
                        ),
                    });
                }
            }
        }
        path
    } else {
        // Reject any non-`Normal` component, then root under plugins_root so callers
        // never receive a CWD-relative path.
        for component in path.components() {
            if !matches!(component, std::path::Component::Normal(_)) {
                return Err(ToolError::InvalidParam {
                    param: "id".into(),
                    message: format!("install path for `{id}` contains invalid path components"),
                });
            }
        }
        canonical_root.join(path)
    };

    // Final containment check. If the path exists, canonicalize it (to catch any
    // symlinks under the path); otherwise compare the lexically-cleaned absolute
    // path directly — `..` components were already rejected above.
    if resolved.exists() {
        let canonical = std::fs::canonicalize(&resolved).map_err(io_internal)?;
        if !canonical.starts_with(&canonical_root) {
            return Err(ToolError::InvalidParam {
                param: "id".into(),
                message: format!("install path for `{id}` resolves outside the plugins directory"),
            });
        }
        Ok(Some(canonical))
    } else {
        if !resolved.starts_with(&canonical_root) {
            return Err(ToolError::InvalidParam {
                param: "id".into(),
                message: format!("install path for `{id}` resolves outside the plugins directory"),
            });
        }
        Ok(Some(resolved))
    }
}

fn copy_tree(source: &Path, dest: &Path) -> Result<(), ToolError> {
    let rd = std::fs::read_dir(source).map_err(io_internal)?;
    for entry in rd.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == ".git" || name == "node_modules" || name == "target" {
            continue;
        }
        let ft = entry.file_type().map_err(|error| ToolError::Sdk {
            sdk_kind: "internal_error".into(),
            message: format!("file_type failed for {}: {error}", path.display()),
        })?;
        if ft.is_symlink() {
            tracing::warn!(
                service = "marketplace",
                event = "copy.skipped",
                path = %path.display(),
                "skipping symlink during copy"
            );
            continue;
        }
        let target = dest.join(entry.file_name());
        if ft.is_dir() {
            std::fs::create_dir_all(&target).map_err(io_internal)?;
            copy_tree(&path, &target)?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(io_internal)?;
            }
            std::fs::copy(&path, &target).map_err(io_internal)?;
        }
    }
    Ok(())
}

fn resolve_relative_path(root: &Path, rel_path: &str) -> Result<PathBuf, ToolError> {
    let candidate = Path::new(rel_path);
    if candidate.is_absolute() {
        return Err(ToolError::InvalidParam {
            message: "path must be relative".into(),
            param: "path".into(),
        });
    }
    for component in candidate.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(ToolError::InvalidParam {
                message: "path must not contain parent-directory traversal".into(),
                param: "path".into(),
            });
        }
    }
    Ok(root.join(candidate))
}

fn io_internal(error: impl std::fmt::Display) -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: error.to_string(),
    }
}

fn detect_lang(p: &Path) -> ArtifactLang {
    match p.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "json" => ArtifactLang::Json,
        "yml" | "yaml" => ArtifactLang::Yaml,
        "md" | "markdown" => ArtifactLang::Markdown,
        "sh" | "bash" => ArtifactLang::Bash,
        "toml" => ArtifactLang::Toml,
        _ => ArtifactLang::Text,
    }
}

async fn sources_add(
    repo: Option<String>,
    url: Option<String>,
    auto_update: Option<bool>,
) -> Result<Value, ToolError> {
    let target = match (repo, url) {
        (Some(r), None) => r,
        (None, Some(u)) => u,
        (Some(_), Some(_)) => {
            return Err(ToolError::InvalidParam {
                param: "repo".into(),
                message: "pass exactly one of `repo` or `url`, not both".into(),
            });
        }
        (None, None) => {
            return Err(ToolError::MissingParam {
                param: "repo".into(),
                message: "one of `repo` or `url` is required".into(),
            });
        }
    };
    let output = tokio::process::Command::new("claude")
        .args(["plugin", "marketplace", "add", &target])
        .output()
        .await
        .map_err(|e| ToolError::Sdk {
            sdk_kind: "internal_error".into(),
            message: format!("spawn `claude plugin marketplace add`: {e}"),
        })?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if !output.status.success() {
        return Err(ToolError::Sdk {
            sdk_kind: "server_error".into(),
            message: format!("`claude plugin marketplace add {target}` failed: {stderr}"),
        });
    }
    if let Some(auto_update) = auto_update {
        persist_marketplace_auto_update(&target, auto_update)?;
    }
    Ok(serde_json::json!({
        "ok": true,
        "target": target,
        "stdout": stdout,
        "stderr": stderr,
    }))
}

fn persist_marketplace_auto_update(target: &str, auto_update: bool) -> Result<(), ToolError> {
    let path = client::plugins_root()?.join("known_marketplaces.json");
    if !path.exists() {
        return Ok(());
    }

    let mut value = read_json(&path)?;
    let Some(entries) = value.as_object_mut() else {
        return Ok(());
    };

    let mut changed = false;
    for (marketplace_id, entry) in entries {
        let Some(entry_obj) = entry.as_object_mut() else {
            continue;
        };
        let source = entry_obj.get("source").and_then(Value::as_object);
        let repo = source
            .and_then(|source| source.get("repo"))
            .and_then(Value::as_str);
        let url = source
            .and_then(|source| source.get("url"))
            .and_then(Value::as_str);
        if marketplace_id == target || repo == Some(target) || url == Some(target) {
            entry_obj.insert("autoUpdate".to_string(), Value::Bool(auto_update));
            changed = true;
        }
    }

    if !changed {
        return Ok(());
    }

    let bytes = serde_json::to_vec_pretty(&value).map_err(io_internal)?;
    std::fs::write(path, bytes).map_err(io_internal)
}

async fn plugin_shell(verb: &'static str, id: &str) -> Result<Value, ToolError> {
    parse_plugin_id(id)?;
    let output = tokio::process::Command::new("claude")
        .args(["plugin", verb, id])
        .output()
        .await
        .map_err(|e| ToolError::Sdk {
            sdk_kind: "internal_error".into(),
            message: format!("spawn `claude plugin {verb}`: {e}"),
        })?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if !output.status.success() {
        return Err(ToolError::Sdk {
            sdk_kind: "server_error".into(),
            message: format!("`claude plugin {verb} {id}` failed: {stderr}"),
        });
    }
    Ok(serde_json::json!({
        "ok": true,
        "id": id,
        "stdout": stdout,
        "stderr": stderr,
    }))
}

#[derive(Debug, Clone, Serialize)]
struct CherryPickDeviceResult {
    device_id: String,
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CherryPickResult {
    results: Vec<CherryPickDeviceResult>,
}

async fn plugin_cherry_pick(
    cp: params::CherryPickParams,
    port: &dyn client::DeviceRpcPort,
) -> Result<Value, ToolError> {
    let rpc_params = serde_json::json!({
        "plugin_id": cp.plugin_id,
        "components": cp.components,
        "scope": cp.scope,
        "project_path": cp.project_path,
    });

    let mut results = Vec::with_capacity(cp.device_ids.len());
    for device_id in &cp.device_ids {
        let outcome = port
            .send_rpc(device_id, "marketplace.install_components", rpc_params.clone())
            .await;
        results.push(match outcome {
            Ok(_) => CherryPickDeviceResult { device_id: device_id.clone(), ok: true, message: None },
            Err(e) => {
                let msg = match &e {
                    ToolError::Sdk { message, .. } => message.clone(),
                    _ => "device RPC failed".into(),
                };
                CherryPickDeviceResult { device_id: device_id.clone(), ok: false, message: Some(msg) }
            }
        });
    }

    to_json(CherryPickResult { results })
}

fn join_err(e: tokio::task::JoinError) -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: format!("spawn_blocking join error: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;
    use tokio::runtime::Builder;

    fn with_home<T>(home: &Path, run: impl FnOnce() -> T) -> T {
        client::with_test_plugins_root(home, run)
    }

    fn dispatch_with_home(home: &Path, action: &str, params: Value) -> Result<Value, ToolError> {
        with_home(home, || {
            Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async { dispatch(action, params).await })
        })
    }

    fn seed_marketplace(home: &Path) {
        let plugins = home.join(".claude").join("plugins");
        std::fs::create_dir_all(plugins.join("marketplaces").join("demo-market").join("demo-plugin")).unwrap();
        std::fs::write(
            plugins.join("known_marketplaces.json"),
            json!({
                "demo-market": {
                    "source": { "source": "github", "repo": "demo/demo-market" },
                    "autoUpdate": false,
                    "lastUpdated": "2026-04-22T00:00:00Z",
                    "installLocation": plugins.join("marketplaces").join("demo-market")
                }
            })
            .to_string(),
        )
        .unwrap();
        std::fs::write(
            plugins.join("marketplaces").join("demo-market").join("marketplace.json"),
            json!({
                "name": "Demo Market",
                "plugins": [{ "name": "demo-plugin", "version": "1.0.0", "description": "Demo" }]
            })
            .to_string(),
        )
        .unwrap();
        std::fs::write(
            plugins.join("marketplaces").join("demo-market").join("demo-plugin").join("plugin.json"),
            r#"{"name":"demo-plugin"}"#,
        )
        .unwrap();
    }

    fn seed_installed(home: &Path) -> PathBuf {
        let plugins = home.join(".claude").join("plugins");
        let install_path = plugins.join("installed").join("demo-plugin");
        std::fs::create_dir_all(&install_path).unwrap();
        std::fs::write(install_path.join("plugin.json"), r#"{"name":"demo-plugin","version":"0.9.0"}"#).unwrap();
        std::fs::write(
            plugins.join("installed_plugins.json"),
            json!({
                "plugins": {
                    "demo-plugin@demo-market": [{
                        "installPath": install_path,
                        "installedAt": "2026-04-22T00:00:00Z",
                        "lastUpdated": "2026-04-22T00:00:00Z"
                    }]
                }
            })
            .to_string(),
        )
        .unwrap();
        install_path
    }

    #[test]
    fn workspace_action_creates_workspace_and_preserves_dotfiles() {
        let dir = tempdir().unwrap();
        with_home(dir.path(), || {
            seed_marketplace(dir.path());
            let source = dir.path().join(".claude").join("plugins").join("marketplaces").join("demo-market").join("demo-plugin");
            std::fs::create_dir_all(source.join(".claude-plugin")).unwrap();
            std::fs::write(source.join(".claude-plugin").join("plugin.json"), r#"{"name":"demo"}"#).unwrap();
        });

        let response =
            dispatch_with_home(dir.path(), "plugin.workspace", json!({ "id": "demo-plugin@demo-market" })).unwrap();

        let files = response.get("files").and_then(Value::as_array).unwrap();
        assert!(files.iter().any(|file| file.get("path").and_then(Value::as_str) == Some(".claude-plugin/plugin.json")));
    }

    #[test]
    fn save_action_writes_workspace_file() {
        let dir = tempdir().unwrap();
        with_home(dir.path(), || {
            seed_marketplace(dir.path());
        });

        dispatch_with_home(
            dir.path(),
            "plugin.save",
            json!({ "id": "demo-plugin@demo-market", "path": "plugin.json", "content": "{\"name\":\"edited\"}" }),
        )
        .unwrap();

        let saved = dir
            .path()
            .join(".claude")
            .join("plugins")
            .join("workspaces")
            .join("demo-plugin@demo-market")
            .join("plugin.json");
        assert_eq!(std::fs::read_to_string(saved).unwrap(), "{\"name\":\"edited\"}");
    }

    #[test]
    fn deploy_action_syncs_workspace_to_installed_target() {
        let dir = tempdir().unwrap();
        let install_path = with_home(dir.path(), || {
            seed_marketplace(dir.path());
            seed_installed(dir.path())
        });

        dispatch_with_home(
            dir.path(),
            "plugin.save",
            json!({ "id": "demo-plugin@demo-market", "path": "plugin.json", "content": "{\"name\":\"deployed\"}" }),
        )
        .unwrap();

        let deployed =
            dispatch_with_home(dir.path(), "plugin.deploy", json!({ "id": "demo-plugin@demo-market" })).unwrap();

        assert_eq!(
            std::fs::read_to_string(install_path.join("plugin.json")).unwrap(),
            "{\"name\":\"deployed\"}"
        );
        assert!(deployed.get("changed").and_then(Value::as_array).unwrap().iter().any(|item| item == "plugin.json"));
    }

    #[test]
    fn deploy_preview_reports_changed_and_removed_files() {
        let dir = tempdir().unwrap();
        with_home(dir.path(), || {
            seed_marketplace(dir.path());
            let install_path = seed_installed(dir.path());
            std::fs::write(install_path.join("stale.txt"), "obsolete").unwrap();
        });

        dispatch_with_home(
            dir.path(),
            "plugin.save",
            json!({ "id": "demo-plugin@demo-market", "path": "plugin.json", "content": "{\"name\":\"previewed\"}" }),
        )
        .unwrap();

        let preview = dispatch_with_home(
            dir.path(),
            "plugin.deploy.preview",
            json!({ "id": "demo-plugin@demo-market" }),
        )
        .unwrap();

        assert!(preview.get("changed").and_then(Value::as_array).unwrap().iter().any(|item| item == "plugin.json"));
        assert!(preview.get("removed").and_then(Value::as_array).unwrap().iter().any(|item| item == "stale.txt"));
        assert!(preview.get("entries").and_then(Value::as_array).unwrap().iter().any(|entry| {
            entry.get("path").and_then(Value::as_str) == Some("plugin.json")
                && entry.get("afterContent").and_then(Value::as_str) == Some("{\"name\":\"previewed\"}")
        }));
    }

    /// Seed installed_plugins.json with an arbitrary `installPath` string for `demo-plugin@demo-market`.
    fn seed_installed_with_path(home: &Path, install_path: &str) {
        let plugins = home.join(".claude").join("plugins");
        std::fs::create_dir_all(&plugins).unwrap();
        std::fs::write(
            plugins.join("installed_plugins.json"),
            json!({
                "plugins": {
                    "demo-plugin@demo-market": [{
                        "installPath": install_path,
                        "installedAt": "2026-04-22T00:00:00Z",
                        "lastUpdated": "2026-04-22T00:00:00Z"
                    }]
                }
            })
            .to_string(),
        )
        .unwrap();
    }

    #[test]
    fn installed_target_rejects_empty_install_path() {
        let dir = tempdir().unwrap();
        let err = with_home(dir.path(), || {
            seed_installed_with_path(dir.path(), "");
            installed_target_for_plugin("demo-plugin@demo-market").unwrap_err()
        });
        match err {
            ToolError::InvalidParam { message, .. } => assert!(message.contains("empty"), "{message}"),
            other => panic!("expected InvalidParam, got {other:?}"),
        }
    }

    #[test]
    fn installed_target_rejects_absolute_path_with_parent_traversal() {
        let dir = tempdir().unwrap();
        let plugins_root = dir.path().join(".claude").join("plugins");
        std::fs::create_dir_all(&plugins_root).unwrap();
        let escape = format!("{}/../../etc/passwd", plugins_root.display());
        let err = with_home(dir.path(), || {
            seed_installed_with_path(dir.path(), &escape);
            installed_target_for_plugin("demo-plugin@demo-market").unwrap_err()
        });
        match err {
            ToolError::InvalidParam { message, .. } => {
                assert!(message.contains("invalid") || message.contains("outside"), "{message}");
            }
            other => panic!("expected InvalidParam, got {other:?}"),
        }
    }

    #[test]
    fn installed_target_rejects_relative_path_with_parent_traversal() {
        let dir = tempdir().unwrap();
        let plugins_root = dir.path().join(".claude").join("plugins");
        std::fs::create_dir_all(&plugins_root).unwrap();
        let err = with_home(dir.path(), || {
            seed_installed_with_path(dir.path(), "../../etc/passwd");
            installed_target_for_plugin("demo-plugin@demo-market").unwrap_err()
        });
        match err {
            ToolError::InvalidParam { message, .. } => {
                assert!(message.contains("invalid"), "{message}");
            }
            other => panic!("expected InvalidParam, got {other:?}"),
        }
    }

    #[test]
    fn installed_target_rejects_absolute_path_outside_plugins_root() {
        let dir = tempdir().unwrap();
        let plugins_root = dir.path().join(".claude").join("plugins");
        std::fs::create_dir_all(&plugins_root).unwrap();
        let err = with_home(dir.path(), || {
            seed_installed_with_path(dir.path(), "/tmp/evil-plugin");
            installed_target_for_plugin("demo-plugin@demo-market").unwrap_err()
        });
        match err {
            ToolError::InvalidParam { message, .. } => assert!(message.contains("outside"), "{message}"),
            other => panic!("expected InvalidParam, got {other:?}"),
        }
    }

    #[test]
    fn installed_target_roots_relative_path_under_plugins_root() {
        let dir = tempdir().unwrap();
        let plugins_root = dir.path().join(".claude").join("plugins");
        std::fs::create_dir_all(&plugins_root).unwrap();
        let target = with_home(dir.path(), || {
            seed_installed_with_path(dir.path(), "installed/demo-plugin");
            installed_target_for_plugin("demo-plugin@demo-market").unwrap()
        })
        .unwrap();
        let canonical_root = std::fs::canonicalize(&plugins_root).unwrap();
        assert!(
            target.starts_with(&canonical_root),
            "{target:?} must be rooted under {canonical_root:?}"
        );
        assert!(target.ends_with("installed/demo-plugin"), "{target:?}");
    }
}
