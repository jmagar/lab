#![allow(dead_code)]

use std::path::Path;

use lab_apis::marketplace::{PluginComponent, PluginComponentKind, PluginManifestSummary};
use serde_json::{Map, Value};

/// Replace the user's home-directory prefix with literal `~`.
///
/// lab-zxx5.27: promoted to `dispatch::helpers::redact_home` so the `node/`
/// install paths can call it without reaching into this marketplace module.
/// This is a thin re-export wrapper; the canonical implementation lives in
/// `dispatch/helpers.rs`.
pub use crate::dispatch::helpers::redact_home;

pub fn manifest_summary_from_marketplace_plugin(
    plugin_json: &Value,
) -> Option<PluginManifestSummary> {
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

pub fn components_from_manifest_and_layout(
    root: Option<&Path>,
    manifest: Option<&Value>,
) -> Vec<PluginComponent> {
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
        collect_component_array(obj, "lspServers", PluginComponentKind::LspServers, out);
        collect_component_array(obj, "lsp_servers", PluginComponentKind::LspServers, out);
        collect_component_array(obj, "commands", PluginComponentKind::Commands, out);
        collect_component_array(obj, "agents", PluginComponentKind::Agents, out);
        collect_component_array(obj, "assets", PluginComponentKind::Assets, out);
        collect_component_array(obj, "hooks", PluginComponentKind::Hooks, out);
        collect_component_array(obj, "monitors", PluginComponentKind::Monitors, out);
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
        ("monitors", PluginComponentKind::Monitors),
        ("bin", PluginComponentKind::Bin),
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
            let name = entry.file_name().to_string_lossy().into_owned();
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

    collect_component_file(root, ".mcp.json", PluginComponentKind::McpServers, out);
    collect_component_file(root, ".lsp.json", PluginComponentKind::LspServers, out);
    collect_component_file(root, "settings.json", PluginComponentKind::Settings, out);
}

fn path_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
        .to_string()
}

fn collect_component_file(
    root: &Path,
    rel_path: &str,
    kind: PluginComponentKind,
    out: &mut Vec<PluginComponent>,
) {
    if root.join(rel_path).exists() {
        out.push(PluginComponent {
            kind,
            path: rel_path.into(),
            name: path_name(rel_path),
            metadata: None,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn components_from_layout_matches_claude_plugin_component_locations() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();

        std::fs::create_dir_all(root.join("skills/review")).expect("skill dir");
        std::fs::write(
            root.join("skills/review/SKILL.md"),
            "---\ndescription: Review\n---",
        )
        .expect("skill");
        std::fs::create_dir_all(root.join("commands")).expect("commands dir");
        std::fs::write(root.join("commands/ship.md"), "Ship").expect("command");
        std::fs::create_dir_all(root.join("agents")).expect("agents dir");
        std::fs::write(root.join("agents/reviewer.md"), "Agent").expect("agent");
        std::fs::create_dir_all(root.join("hooks")).expect("hooks dir");
        std::fs::write(root.join("hooks/hooks.json"), "{}").expect("hooks");
        std::fs::create_dir_all(root.join("monitors")).expect("monitors dir");
        std::fs::write(root.join("monitors/monitors.json"), "[]").expect("monitors");
        std::fs::create_dir_all(root.join("bin")).expect("bin dir");
        std::fs::write(root.join("bin/tool"), "#!/bin/sh\n").expect("bin");
        std::fs::write(root.join(".mcp.json"), "{\"mcpServers\":{}}").expect("mcp");
        std::fs::write(root.join(".lsp.json"), "{}").expect("lsp");
        std::fs::write(root.join("settings.json"), "{}").expect("settings");

        let components = components_from_manifest_and_layout(Some(root), None);
        let observed: std::collections::HashSet<_> = components
            .iter()
            .map(|component| (component.kind, component.path.as_str()))
            .collect();

        assert!(observed.contains(&(PluginComponentKind::Skills, "skills/review")));
        assert!(observed.contains(&(PluginComponentKind::Commands, "commands/ship.md")));
        assert!(observed.contains(&(PluginComponentKind::Agents, "agents/reviewer.md")));
        assert!(observed.contains(&(PluginComponentKind::Hooks, "hooks/hooks.json")));
        assert!(observed.contains(&(PluginComponentKind::Monitors, "monitors/monitors.json")));
        assert!(observed.contains(&(PluginComponentKind::Bin, "bin/tool")));
        assert!(observed.contains(&(PluginComponentKind::McpServers, ".mcp.json")));
        assert!(observed.contains(&(PluginComponentKind::LspServers, ".lsp.json")));
        assert!(observed.contains(&(PluginComponentKind::Settings, "settings.json")));
    }

    #[test]
    fn components_from_manifest_accepts_claude_component_fields() {
        let manifest = serde_json::json!({
            "skills": ["skills/review"],
            "commands": ["commands/ship.md"],
            "agents": ["agents/reviewer.md"],
            "mcpServers": ".mcp.json",
            "lspServers": ".lsp.json",
            "monitors": "monitors/monitors.json"
        });

        let components = components_from_manifest_and_layout(None, Some(&manifest));
        let observed: std::collections::HashSet<_> = components
            .iter()
            .map(|component| (component.kind, component.path.as_str()))
            .collect();

        assert!(observed.contains(&(PluginComponentKind::Skills, "skills/review")));
        assert!(observed.contains(&(PluginComponentKind::Commands, "commands/ship.md")));
        assert!(observed.contains(&(PluginComponentKind::Agents, "agents/reviewer.md")));
        assert!(observed.contains(&(PluginComponentKind::McpServers, ".mcp.json")));
        assert!(observed.contains(&(PluginComponentKind::LspServers, ".lsp.json")));
        assert!(observed.contains(&(PluginComponentKind::Monitors, "monitors/monitors.json")));
    }
}
