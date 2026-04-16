use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredMcpConfigFile {
    pub source: String,
    pub path: PathBuf,
    pub modified_unix_secs: u64,
    pub content_hash: String,
    pub servers: BTreeMap<String, serde_json::Value>,
}

pub fn discover_ai_cli_configs(home: &Path) -> Result<Vec<DiscoveredMcpConfigFile>> {
    let mut discovered = Vec::new();

    if let Some(file) = scan_json_config("claude", &home.join(".claude.json"))? {
        discovered.push(file);
    }

    if let Some(file) = scan_codex_config("codex", &home.join(".codex/config.toml"))? {
        discovered.push(file);
    }

    if let Some(file) = scan_json_config("gemini", &home.join(".gemini/settings.json"))? {
        discovered.push(file);
    }

    Ok(discovered)
}

fn scan_json_config(source: &str, path: &Path) -> Result<Option<DiscoveredMcpConfigFile>> {
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let parsed: serde_json::Value =
        serde_json::from_slice(&raw).with_context(|| format!("parse {}", path.display()))?;
    let servers = parsed
        .get("mcpServers")
        .and_then(serde_json::Value::as_object)
        .map(|servers| {
            servers
                .iter()
                .map(|(name, value)| (name.clone(), value.clone()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    Ok(Some(build_discovered_file(source, path, &raw, servers)?))
}

fn scan_codex_config(source: &str, path: &Path) -> Result<Option<DiscoveredMcpConfigFile>> {
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let parsed: toml::Value =
        toml::from_slice(&raw).with_context(|| format!("parse {}", path.display()))?;
    let servers = parsed
        .get("mcp_servers")
        .and_then(toml::Value::as_table)
        .map(|servers| {
            servers
                .iter()
                .map(|(name, value)| {
                    (
                        name.clone(),
                        serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
                    )
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    Ok(Some(build_discovered_file(source, path, &raw, servers)?))
}

fn build_discovered_file(
    source: &str,
    path: &Path,
    raw: &[u8],
    servers: BTreeMap<String, serde_json::Value>,
) -> Result<DiscoveredMcpConfigFile> {
    let modified_unix_secs = fs::metadata(path)
        .with_context(|| format!("metadata {}", path.display()))?
        .modified()
        .with_context(|| format!("modified time {}", path.display()))?
        .duration_since(UNIX_EPOCH)
        .with_context(|| format!("unix timestamp {}", path.display()))?
        .as_secs();

    let content_hash = format!("{:x}", Sha256::digest(raw));

    Ok(DiscoveredMcpConfigFile {
        source: source.to_string(),
        path: path.to_path_buf(),
        modified_unix_secs,
        content_hash,
        servers,
    })
}
