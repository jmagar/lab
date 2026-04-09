//! MCP dispatch for the Radarr tool.
//!
//! Split into focused sub-modules by resource group. This entry point
//! assembles the combined `ACTIONS` catalog and delegates dispatch to
//! the appropriate sub-module.

mod calendar;
mod commands;
mod config;
pub mod helpers;
mod history;
mod movies;
mod queue;
mod system;

use lab_apis::core::action::ActionSpec;
use serde_json::Value;

use crate::services::error::ToolError;

// Re-export helpers used by other modules (e.g. cli/radarr.rs, cli/health.rs).
pub use helpers::client_from_env;

/// Combined action catalog for the Radarr tool.
///
/// Built once at first access by concatenating each sub-module's `ACTIONS`
/// slice. The resulting `Vec` is leaked to produce a `&'static [ActionSpec]`
/// compatible with `RegisteredService::actions`.
pub fn actions() -> &'static [ActionSpec] {
    static ACTIONS: std::sync::LazyLock<&'static [ActionSpec]> =
        std::sync::LazyLock::new(|| {
            let mut all = vec![ActionSpec {
                name: "help",
                description: "Show this action catalog",
                destructive: false,
                returns: "Catalog",
                params: &[],
            }];
            all.extend_from_slice(system::ACTIONS);
            all.extend_from_slice(movies::ACTIONS);
            all.extend_from_slice(queue::ACTIONS);
            all.extend_from_slice(calendar::ACTIONS);
            all.extend_from_slice(commands::ACTIONS);
            all.extend_from_slice(history::ACTIONS);
            all.extend_from_slice(config::ACTIONS);
            Vec::leak(all)
        });
    &ACTIONS
}


/// Dispatch one MCP call against the Radarr tool.
///
/// # Errors
/// Returns a `ToolError` on unknown actions, missing params, or SDK failures.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(serde_json::json!({
            "service": "radarr",
            "actions": actions().iter().map(|a| serde_json::json!({
                "name": a.name,
                "description": a.description,
                "destructive": a.destructive,
                "returns": a.returns,
                "params": a.params.iter().map(|p| serde_json::json!({
                    "name": p.name,
                    "type": p.ty,
                    "required": p.required,
                    "description": p.description,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
        })),

        // System
        a if a.starts_with("system.") => system::dispatch(action, params).await,

        // Movies
        a if a.starts_with("movie.") => movies::dispatch(action, params).await,

        // Queue
        a if a.starts_with("queue.") => queue::dispatch(action, params).await,

        // Calendar
        "calendar.list" => calendar::dispatch(action, params).await,

        // Commands
        a if a.starts_with("command.") => commands::dispatch(action, params).await,

        // History & blocklist
        "history.list" | "blocklist.list" => history::dispatch(action, params).await,

        // Everything else: releases, indexers, quality, root-folders, tags,
        // download-clients, config, notifications, import-lists, language,
        // metadata, filesystem.
        "release.search"
        | "indexer.list" | "indexer.test"
        | "quality-profile.list" | "quality-definition.list"
        | "root-folder.list"
        | "tag.list" | "tag.detail-list"
        | "download-client.list" | "download-client.test"
        | "remote-path-mapping.list"
        | "config.host" | "config.naming" | "config.ui"
        | "notification.list" | "notification.test"
        | "import-list.list" | "import-list.exclusion-list"
        | "language.list"
        | "metadata.list"
        | "filesystem.list" => config::dispatch(action, params).await,

        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action `{unknown}` for service `radarr`"),
            valid: actions().iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

#[cfg(test)]
mod tests {
    use lab_apis::core::error::ApiError;
    use lab_apis::radarr::error::RadarrError;

    use super::*;

    // ── From<RadarrError> for ToolError ──────────────────────────────────────

    #[test]
    fn sdk_error_from_api_auth() {
        let e: ToolError = RadarrError::Api(ApiError::Auth).into();
        assert_eq!(e.kind(), "auth_failed");
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["kind"], "auth_failed");
        assert!(v.get("sdk_kind").is_none());
    }

    #[test]
    fn sdk_error_from_api_not_found() {
        let e: ToolError = RadarrError::Api(ApiError::NotFound).into();
        assert_eq!(e.kind(), "not_found");
    }

    #[test]
    fn sdk_error_from_radarr_not_found() {
        let e: ToolError = RadarrError::NotFound {
            kind: "movie",
            id: 42,
        }
        .into();
        assert_eq!(e.kind(), "not_found");
    }

    #[test]
    fn sdk_error_from_network() {
        let e: ToolError = RadarrError::Api(ApiError::Network("timeout".into())).into();
        assert_eq!(e.kind(), "network_error");
    }

    // ── unknown_action shape ─────────────────────────────────────────────────

    #[test]
    fn unknown_action_includes_valid_list() {
        let e = ToolError::UnknownAction {
            message: "unknown action `bogus`".into(),
            valid: actions().iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        };
        let v = serde_json::to_value(&e).unwrap();
        assert_eq!(v["kind"], "unknown_action");
        let valid = v["valid"].as_array().unwrap();
        assert!(
            valid.iter().any(|s| s == "help"),
            "help must be in valid list"
        );
        assert!(
            valid.iter().any(|s| s == "movie.list"),
            "movie.list must be in valid list"
        );
    }

    #[test]
    fn action_count_preserved() {
        // Original monolith had 40 ActionSpecs (including help).
        assert_eq!(
            actions().len(),
            40,
            "expected 40 actions (help + 39 resource actions), got {}",
            actions().len()
        );
    }

    #[test]
    fn no_duplicate_action_names() {
        let names: Vec<&str> = actions().iter().map(|a| a.name).collect();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(
            names.len(),
            sorted.len(),
            "duplicate action names found in ACTIONS"
        );
    }
}
