use std::collections::HashMap;
use std::path::{Path, PathBuf};

use lab_apis::marketplace::{Artifact, ArtifactLang, Marketplace, Plugin, PluginSource};
use serde_json::{Map, Value};

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, optional_str, require_str, to_json};
use crate::dispatch::marketplace::catalog::ACTIONS;
use crate::dispatch::marketplace::params::parse_plugin_id;

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    let start = std::time::Instant::now();
    let result = dispatch_inner(action, params).await;
    let elapsed_ms = start.elapsed().as_millis();

    match &result {
        Ok(_) => tracing::info!(
            surface = "mcp",
            service = "marketplace",
            action,
            elapsed_ms,
            "dispatch ok"
        ),
        Err(err) => {
            let k = err.kind();
            if err.is_internal() {
                tracing::error!(surface = "mcp", service = "marketplace", action, elapsed_ms, kind = k, "dispatch error");
            } else {
                tracing::warn!(surface = "mcp", service = "marketplace", action, elapsed_ms, kind = k, "dispatch error");
            }
        }
    }
    result
}

async fn dispatch_inner(action: &str, params: Value) -> Result<Value, ToolError> {
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
            sources_add(repo, url).await
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
        "plugin.install" => {
            let id = require_str(&params, "id")?.to_string();
            plugin_shell("install", &id).await
        }
        "plugin.uninstall" => {
            let id = require_str(&params, "id")?.to_string();
            plugin_shell("uninstall", &id).await
        }
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action `marketplace.{unknown}`"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

// ---------- paths ----------

fn plugins_root() -> Result<PathBuf, ToolError> {
    let home = std::env::var_os("HOME").ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "HOME env var not set".into(),
    })?;
    Ok(PathBuf::from(home).join(".claude").join("plugins"))
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
    let path = plugins_root()?.join("known_marketplaces.json");
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

fn load_installed() -> Result<HashMap<String, InstalledRecord>, ToolError> {
    let path = plugins_root()?.join("installed_plugins.json");
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
            let install_loc = plugins_root()?
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
        let install_loc = plugins_root()?.join("marketplaces").join(&mkt);
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
        let installed = load_installed()?;
        let Some(rec) = installed.get(&id_owned) else {
            return Err(ToolError::Sdk {
                sdk_kind: "not_found".into(),
                message: format!("plugin `{id_owned}` is not installed"),
            });
        };
        walk_artifacts(&rec.install_path, &rec.install_path)
    })
    .await
    .map_err(join_err)??;
    to_json(artifacts)
}

fn walk_artifacts(root: &Path, dir: &Path) -> Result<Vec<Artifact>, ToolError> {
    let mut out = Vec::new();
    let rd = std::fs::read_dir(dir).map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: format!("read_dir {}: {e}", dir.display()),
    })?;
    for entry in rd.flatten() {
        let p = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }
        if p.is_dir() {
            out.extend(walk_artifacts(root, &p)?);
            continue;
        }
        if out.len() >= 200 {
            break;
        }
        let meta = entry.metadata().ok();
        if let Some(m) = meta {
            if m.len() > 256 * 1024 {
                continue;
            }
        }
        let rel = p
            .strip_prefix(root)
            .unwrap_or(&p)
            .to_string_lossy()
            .into_owned();
        let lang = detect_lang(&p);
        let content = std::fs::read_to_string(&p).unwrap_or_default();
        out.push(Artifact {
            path: rel,
            lang,
            content,
        });
    }
    Ok(out)
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

async fn sources_add(repo: Option<String>, url: Option<String>) -> Result<Value, ToolError> {
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
    Ok(serde_json::json!({
        "ok": true,
        "target": target,
        "stdout": stdout,
        "stderr": stderr,
    }))
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

fn join_err(e: tokio::task::JoinError) -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: format!("spawn_blocking join error: {e}"),
    }
}
