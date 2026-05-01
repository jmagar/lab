use std::fs;
use std::path::PathBuf;

use lab_apis::notebooklm::NotebookLmClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

pub fn client_from_env() -> Option<NotebookLmClient> {
    storage_json_from_env()
        .ok()
        .and_then(|json| NotebookLmClient::from_storage_json(&json).ok())
}

pub fn require_client() -> Result<NotebookLmClient, ToolError> {
    let storage_json = storage_json_from_env().map_err(|message| ToolError::Sdk {
        sdk_kind: "auth_failed".to_string(),
        message,
    })?;
    NotebookLmClient::from_storage_json(&storage_json).map_err(ToolError::from)
}

pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "auth_failed".to_string(),
        message:
            "NotebookLM auth is not configured; set NOTEBOOKLM_AUTH_JSON or run notebooklm login"
                .to_string(),
    }
}

fn storage_json_from_env() -> Result<String, String> {
    if let Some(json) = env_non_empty("NOTEBOOKLM_AUTH_JSON") {
        return Ok(json);
    }

    let paths = storage_paths();
    for path in &paths {
        if path.exists() {
            return fs::read_to_string(path).map_err(|e| {
                format!(
                    "NotebookLM storage file not readable at {}: {e}",
                    path.display()
                )
            });
        }
    }
    let path = paths
        .first()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("storage_state.json"));
    fs::read_to_string(&path).map_err(|e| {
        format!(
            "NotebookLM storage file not readable at {}: {e}",
            path.display()
        )
    })
}

fn storage_paths() -> Vec<PathBuf> {
    if let Some(path) = env_non_empty("NOTEBOOKLM_STORAGE") {
        return vec![PathBuf::from(path)];
    }
    default_storage_paths()
}

fn default_storage_paths() -> Vec<PathBuf> {
    let base = env_non_empty("NOTEBOOKLM_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_default()
                .join(".notebooklm")
        });
    let profile = env_non_empty("NOTEBOOKLM_PROFILE")
        .or_else(|| default_profile_from_config(&base))
        .unwrap_or_else(|| "default".to_string());
    storage_paths_for_profile(&base, &profile)
}

fn storage_paths_for_profile(base: &std::path::Path, profile: &str) -> Vec<PathBuf> {
    let mut paths = vec![
        base.join("profiles")
            .join(profile)
            .join("storage_state.json"),
    ];
    if profile == "default" {
        paths.push(base.join("storage_state.json"));
    }
    paths
}

fn default_profile_from_config(base: &std::path::Path) -> Option<String> {
    let config = fs::read_to_string(base.join("config.json")).ok()?;
    let value: Value = serde_json::from_str(&config).ok()?;
    value
        .get("default_profile")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_default_profile_from_config() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("config.json"),
            r#"{"default_profile":"research"}"#,
        )
        .unwrap();
        assert_eq!(
            default_profile_from_config(dir.path()).as_deref(),
            Some("research")
        );
    }

    #[test]
    fn default_profile_paths_include_legacy_storage() {
        let base = PathBuf::from("/tmp/notebooklm-home");
        let paths = storage_paths_for_profile(&base, "default");
        assert_eq!(
            paths,
            vec![
                PathBuf::from("/tmp/notebooklm-home/profiles/default/storage_state.json"),
                PathBuf::from("/tmp/notebooklm-home/storage_state.json"),
            ]
        );
    }
}
