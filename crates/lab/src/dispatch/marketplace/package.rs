use std::path::Path;

use lab_apis::marketplace::{PluginComponent, PluginComponentKind, PluginManifestSummary};
use serde_json::{Map, Value};

/// Replace the user's home-directory prefix with literal `~` so paths emitted
/// in `plugin.list` / `plugin.get` responses don't leak the OS username.
///
/// Security (bead lab-zxx5.14, finding D1): defeats home-dir / username
/// disclosure in authenticated plugin responses without breaking per-runtime
/// subdir visibility (`~/.claude/plugins/` vs `~/.codex/plugins/` still
/// distinguishable). Safe on any input: if `HOME` is unset or the path
/// doesn't sit under it, the input is returned unchanged.
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

pub fn manifest_summary_from_marketplace_plugin(plugin_json: &Value) -> Option<PluginManifestSummary> {
    Some(PluginManifestSummary {
        description: plugin_json
            .get("description")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        version: plugin_json
            .get("version")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        interface: None,
    })
}

pub fn manifest_summary_from_codex_manifest(manifest: &Value) -> Option<PluginManifestSummary> {
    Some(PluginManifestSummary {
        description: manifest
            .get("description")
            .or_else(|| manifest.get("metadata").and_then(|m| m.get("description")))
            .and_then(Value::as_str)
            .map(ToString::to_string),
        version: manifest
            .get("version")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        interface: manifest.get("interface").cloned(),
    })
}

pub fn tags_from_marketplace_plugin(plugin_json: &Value) -> Vec<String> {
    let mut tags = Vec::new();
    for key in ["tags", "keywords"] {
        if let Some(arr) = plugin_json.get(key).and_then(Value::as_array) {
            for value in arr {
                if let Some(tag) = value.as_str() {
                    let tag = tag.to_string();
                    if !tags.contains(&tag) {
                        tags.push(tag);
                    }
                }
            }
        }
    }
    if let Some(category) = plugin_json.get("category").and_then(Value::as_str) {
        let category = category.to_string();
        if !tags.contains(&category) {
            tags.insert(0, category);
        }
    }
    tags
}

pub fn components_from_manifest_and_layout(root: Option<&Path>, manifest: Option<&Value>) -> Vec<PluginComponent> {
    let mut out = Vec::new();
    if let Some(manifest) = manifest {
        collect_components_from_value(manifest, &mut out);
    }
    if let Some(root) = root {
        collect_components_from_layout(root, &mut out);
    }
    out.sort_by(|left, right| left.path.cmp(&right.path));
    out.dedup_by(|left, right| left.kind == right.kind && left.path == right.path);
    out
}

fn collect_components_from_value(manifest: &Value, out: &mut Vec<PluginComponent>) {
    if let Some(obj) = manifest.as_object() {
        collect_component_array(obj, "skills", PluginComponentKind::Skills, out);
        collect_component_array(obj, "apps", PluginComponentKind::Apps, out);
        collect_component_array(obj, "mcpServers", PluginComponentKind::McpServers, out);
        collect_component_array(obj, "mcp_servers", PluginComponentKind::McpServers, out);
        collect_component_array(obj, "commands", PluginComponentKind::Commands, out);
        collect_component_array(obj, "agents", PluginComponentKind::Agents, out);
        collect_component_array(obj, "assets", PluginComponentKind::Assets, out);
        collect_component_array(obj, "hooks", PluginComponentKind::Hooks, out);
    }
}

fn collect_component_array(
    obj: &Map<String, Value>,
    key: &str,
    kind: PluginComponentKind,
    out: &mut Vec<PluginComponent>,
) {
    let Some(value) = obj.get(key) else {
        return;
    };
    match value {
        Value::Array(items) => {
            for item in items {
                if let Some(component) = component_from_value(kind, item) {
                    out.push(component);
                }
            }
        }
        Value::Object(items) => {
            for (name, item) in items {
                if let Some(component) = component_from_object_entry(kind, name, item) {
                    out.push(component);
                }
            }
        }
        Value::String(path) => out.push(PluginComponent {
            kind,
            path: path.clone(),
            name: path_name(path),
            metadata: None,
        }),
        _ => {}
    }
}

fn component_from_value(kind: PluginComponentKind, value: &Value) -> Option<PluginComponent> {
    match value {
        Value::String(path) => Some(PluginComponent {
            kind,
            path: path.clone(),
            name: path_name(path),
            metadata: None,
        }),
        Value::Object(map) => {
            let path = map
                .get("path")
                .or_else(|| map.get("file"))
                .or_else(|| map.get("entry"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            if path.is_empty() {
                return None;
            }
            let name = map
                .get("name")
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .unwrap_or_else(|| path_name(&path));
            Some(PluginComponent {
                kind,
                path,
                name,
                metadata: Some(Value::Object(map.clone())),
            })
        }
        _ => None,
    }
}

fn component_from_object_entry(
    kind: PluginComponentKind,
    name: &str,
    value: &Value,
) -> Option<PluginComponent> {
    match value {
        Value::String(path) => Some(PluginComponent {
            kind,
            path: path.clone(),
            name: name.to_string(),
            metadata: None,
        }),
        Value::Object(map) => {
            let path = map
                .get("path")
                .or_else(|| map.get("file"))
                .or_else(|| map.get("entry"))
                .and_then(Value::as_str)
                .unwrap_or(name)
                .to_string();
            Some(PluginComponent {
                kind,
                path,
                name: name.to_string(),
                metadata: Some(Value::Object(map.clone())),
            })
        }
        _ => None,
    }
}

fn collect_components_from_layout(root: &Path, out: &mut Vec<PluginComponent>) {
    let specs = [
        ("skills", PluginComponentKind::Skills),
        ("apps", PluginComponentKind::Apps),
        ("commands", PluginComponentKind::Commands),
        ("agents", PluginComponentKind::Agents),
        ("assets", PluginComponentKind::Assets),
        ("hooks", PluginComponentKind::Hooks),
    ];

    for (dir_name, kind) in specs {
        let dir = root.join(dir_name);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned();
            let name = entry
                .file_name()
                .to_string_lossy()
                .into_owned();
            out.push(PluginComponent {
                kind,
                path: rel,
                name,
                metadata: None,
            });
        }
    }

    let codex_manifest = root.join(".codex-plugin").join("plugin.json");
    if codex_manifest.exists() {
        out.push(PluginComponent {
            kind: PluginComponentKind::Files,
            path: ".codex-plugin/plugin.json".into(),
            name: "plugin.json".into(),
            metadata: None,
        });
    }
}

fn path_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
        .to_string()
}
