//! `.mcp.json` atomic patcher — safely enables/disables services.
//!
//! Uses a sidecar lock file (`.mcp.json.lock`) so fd-lock mutual exclusion is
//! not escaped by `persist()`'s `rename(2)` call.

use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, bail};
use fd_lock::RwLock;
use serde_json::Value;
use tempfile::NamedTempFile;

/// All valid compiled-in service names. Used for injection prevention.
const VALID_SERVICE_NAMES: &[&str] = &[
    "radarr",
    "sonarr",
    "prowlarr",
    "overseerr",
    "plex",
    "tautulli",
    "sabnzbd",
    "qbittorrent",
    "tailscale",
    "unraid",
    "unifi",
    "arcane",
    "linkding",
    "memos",
    "bytestash",
    "paperless",
    "gotify",
    "apprise",
    "openai",
    "notebooklm",
    "qdrant",
    "tei",
    "extract",
];

/// Atomically enable or disable a service in `.mcp.json`'s `--services` array.
///
/// Uses a sidecar lock file (`.mcp.json.lock`) so fd-lock mutual exclusion is
/// not escaped by `persist()`'s `rename(2)` call.
pub fn patch_mcp_json(path: &Path, service_name: &str, enabled: bool) -> anyhow::Result<()> {
    // 1. Validate service_name — never write arbitrary input into .mcp.json args.
    if !VALID_SERVICE_NAMES.contains(&service_name) {
        bail!("unknown service '{service_name}' — refusing to write to .mcp.json");
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let lock_path = parent.join(".mcp.json.lock");

    // 2. Acquire sidecar lock (never renamed, so flock stays valid across persist).
    let lock_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&lock_path)
        .context("open sidecar lock")?;
    let mut rw = RwLock::new(lock_file);

    // Retry up to 3 times at 100ms backoff before giving up.
    // Each try_write() attempt must be in a separate scope to avoid re-borrowing.
    let _guard = 'lock: {
        for attempt in 0..3u32 {
            match rw.try_write() {
                Ok(g) => break 'lock g,
                Err(_) if attempt < 2 => std::thread::sleep(Duration::from_millis(100)),
                Err(_) => {
                    bail!(".mcp.json is locked — Claude Code may be writing; retry in a moment")
                }
            }
        }
        bail!(".mcp.json is locked — retry in a moment");
    };

    // 3. Stat mtime before read — detect concurrent non-cooperating writers.
    let pre_mtime = path.metadata().ok().and_then(|m| m.modified().ok());

    // 4. If .mcp.json doesn't exist, create a fresh one.
    let content = if path.exists() {
        fs::read_to_string(path).context("read .mcp.json")?
    } else {
        let current_exe = std::env::current_exe().context("current_exe")?;
        return create_mcp_json(path, &current_exe, service_name, enabled);
    };

    // 5. Strict parse — refuse JSONC or invalid JSON.
    let mut root: Value = serde_json::from_str(&content)
        .context("parse .mcp.json — file contains invalid JSON (JSONC not supported)")?;

    // 6. Backup before mutating. Prune old backups.
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let bak_path = parent.join(format!(".mcp.json.bak.{ts}"));
    fs::copy(path, &bak_path).context("backup .mcp.json")?;
    prune_backups(parent)?;

    // 7. Mutate — find or create mcpServers.lab.args --services array.
    patch_services_array(&mut root, service_name, enabled)?;

    // 8. Verify mtime hasn't changed under us.
    let post_mtime = path.metadata().ok().and_then(|m| m.modified().ok());
    if pre_mtime != post_mtime {
        bail!(".mcp.json changed on disk while patching — aborting; reload the TUI");
    }

    // 9. Atomic write: temp in same dir, sync, rename.
    let mut tmp = NamedTempFile::new_in(parent).context("create temp file")?;
    serde_json::to_writer_pretty(&mut tmp, &root).context("serialize .mcp.json")?;
    tmp.as_file().sync_all().context("sync temp file")?;
    tmp.persist(path).context("atomic rename")?;

    Ok(())
}

fn patch_services_array(root: &mut Value, service_name: &str, enabled: bool) -> anyhow::Result<()> {
    let servers = root
        .get_mut("mcpServers")
        .and_then(|s| s.get_mut("lab"))
        .and_then(|l| l.get_mut("args"))
        .and_then(|a| a.as_array_mut())
        .context("mcpServers.lab.args not found")?;

    // Find --services index; collect services after it.
    let services_idx = servers
        .iter()
        .position(|v| v.as_str() == Some("--services"))
        .context("--services flag not found in args")?;

    let mut current: Vec<String> = servers[services_idx + 1..]
        .iter()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();

    if enabled {
        if !current.contains(&service_name.to_owned()) {
            current.push(service_name.to_owned());
        }
    } else {
        current.retain(|s| s != service_name);
    }

    // Dedupe preserving order.
    let mut seen = HashSet::new();
    current.retain(|s| seen.insert(s.clone()));

    // Rebuild args: everything up to and including --services, then new services list.
    let prefix: Vec<Value> = servers[..=services_idx].to_vec();
    let suffix: Vec<Value> = current.into_iter().map(Value::String).collect();
    *servers = prefix.into_iter().chain(suffix).collect();
    Ok(())
}

fn create_mcp_json(
    path: &Path,
    exe: &Path,
    service_name: &str,
    enabled: bool,
) -> anyhow::Result<()> {
    let mut args: Vec<Value> = vec![
        Value::String("mcp".to_owned()),
        Value::String("--services".to_owned()),
    ];
    if enabled {
        args.push(Value::String(service_name.to_owned()));
    }
    let root = serde_json::json!({
        "mcpServers": {
            "lab": {
                "command": exe.display().to_string(),
                "args": args
            }
        }
    });
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = NamedTempFile::new_in(parent)?;
    serde_json::to_writer_pretty(&mut tmp, &root)?;
    tmp.as_file().sync_all()?;
    tmp.persist(path)?;
    Ok(())
}

fn prune_backups(dir: &Path) -> anyhow::Result<()> {
    let mut backups: Vec<_> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|n| n.starts_with(".mcp.json.bak."))
        })
        .collect();
    if backups.len() > 10 {
        backups.sort_by_key(fs::DirEntry::file_name);
        for entry in &backups[..backups.len() - 10] {
            drop(fs::remove_file(entry.path()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use tempfile::TempDir;

    fn make_mcp_json(dir: &TempDir, content: &str) -> PathBuf {
        let path = dir.path().join(".mcp.json");
        fs::write(&path, content).unwrap();
        path
    }

    fn base_json() -> &'static str {
        r#"{
  "mcpServers": {
    "lab": {
      "command": "/usr/local/bin/labby",
      "args": ["mcp", "--services", "radarr"]
    }
  }
}"#
    }

    #[test]
    fn enable_service_appends() {
        let dir = TempDir::new().unwrap();
        let path = make_mcp_json(&dir, base_json());
        patch_mcp_json(&path, "sonarr", true).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let v: Value = serde_json::from_str(&content).unwrap();
        let args = v["mcpServers"]["lab"]["args"].as_array().unwrap();
        let services: Vec<&str> = args
            .iter()
            .skip_while(|a| a.as_str() != Some("--services"))
            .skip(1)
            .filter_map(|a| a.as_str())
            .collect();
        assert!(services.contains(&"radarr"));
        assert!(services.contains(&"sonarr"));
    }

    #[test]
    fn disable_service_removes() {
        let dir = TempDir::new().unwrap();
        let path = make_mcp_json(&dir, base_json());
        patch_mcp_json(&path, "radarr", false).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let v: Value = serde_json::from_str(&content).unwrap();
        let args = v["mcpServers"]["lab"]["args"].as_array().unwrap();
        let services: Vec<&str> = args
            .iter()
            .skip_while(|a| a.as_str() != Some("--services"))
            .skip(1)
            .filter_map(|a| a.as_str())
            .collect();
        assert!(!services.contains(&"radarr"));
    }

    #[test]
    fn dedupe_on_write() {
        let dir = TempDir::new().unwrap();
        let duped = r#"{
  "mcpServers": {
    "lab": {
      "command": "/usr/local/bin/labby",
      "args": ["mcp", "--services", "radarr", "radarr", "sonarr"]
    }
  }
}"#;
        let path = make_mcp_json(&dir, duped);
        patch_mcp_json(&path, "prowlarr", true).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let v: Value = serde_json::from_str(&content).unwrap();
        let args = v["mcpServers"]["lab"]["args"].as_array().unwrap();
        let services: Vec<&str> = args
            .iter()
            .skip_while(|a| a.as_str() != Some("--services"))
            .skip(1)
            .filter_map(|a| a.as_str())
            .collect();
        let radarr_count = services.iter().filter(|&&s| s == "radarr").count();
        assert_eq!(radarr_count, 1, "radarr should be deduped to 1");
        assert!(services.contains(&"prowlarr"));
    }

    #[test]
    fn refuse_invalid_json() {
        let dir = TempDir::new().unwrap();
        let path = make_mcp_json(&dir, "{ not valid json }");
        let result = patch_mcp_json(&path, "radarr", true);
        assert!(result.is_err());
    }

    #[test]
    fn service_name_validation_rejects_traversal() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".mcp.json");
        let result = patch_mcp_json(&path, "../../etc/passwd", true);
        assert!(result.is_err());
        assert!(!path.exists()); // file not created
    }

    #[test]
    fn service_name_validation_rejects_unknown() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".mcp.json");
        let result = patch_mcp_json(&path, "notaservice", true);
        assert!(result.is_err());
        assert!(!path.exists());
    }

    #[test]
    fn creates_file_when_missing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".mcp.json");
        assert!(!path.exists());
        patch_mcp_json(&path, "radarr", true).unwrap();
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        let v: Value = serde_json::from_str(&content).unwrap();
        let args = v["mcpServers"]["lab"]["args"].as_array().unwrap();
        let has_radarr = args.iter().any(|a| a.as_str() == Some("radarr"));
        assert!(has_radarr);
    }
}
