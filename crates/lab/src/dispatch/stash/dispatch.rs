//! Top-level action router for the `stash` dispatch service.
//!
//! `dispatch()` handles the two built-in meta-actions (`help`, `schema`) and
//! delegates all service-specific actions to `dispatch_with_store()`.
//!
//! # Observability
//!
//! `dispatch()` emits one structured event per call at the `INFO` level on
//! success, `WARN` for caller errors, and `ERROR` for internal failures.
//! Fields follow the standard dispatch schema from `docs/OBSERVABILITY.md`:
//! `surface`, `service`, `action`, `elapsed_ms`, and `kind` (errors only).
//!
//! Note: `surface` is hardcoded to `"mcp"` here — the shared dispatch layer
//! does not yet thread surface context through from the calling surface.

use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str};

use super::catalog::ACTIONS;
use super::client::require_stash_root;
use super::params::{
    parse_create_params, parse_deploy_params, parse_export_params, parse_get_params,
    parse_import_params, parse_link_params, parse_provider_sync_params, parse_revisions_params,
    parse_save_params, parse_target_add_params, parse_target_remove_params, parse_workspace_params,
};
use super::service;
use super::store::StashStore;

/// Top-level MCP/CLI entry point.
///
/// Handles `help` and `schema` directly, then constructs the store and
/// delegates to [`dispatch_with_store`].
///
/// Emits structured dispatch events — see module docs for field details.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    let start = std::time::Instant::now();
    let result = dispatch_inner(action, params).await;
    let elapsed_ms = start.elapsed().as_millis();

    match &result {
        Ok(_) => tracing::info!(
            surface = "mcp",
            service = "stash",
            action,
            elapsed_ms,
            "dispatch ok"
        ),
        Err(err) => {
            if err.is_internal() {
                tracing::error!(
                    surface = "mcp",
                    service = "stash",
                    action,
                    elapsed_ms,
                    kind = err.kind(),
                    "dispatch error"
                );
            } else {
                tracing::warn!(
                    surface = "mcp",
                    service = "stash",
                    action,
                    elapsed_ms,
                    kind = err.kind(),
                    "dispatch error"
                );
            }
        }
    }
    result
}

async fn dispatch_inner(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("stash", ACTIONS)),
        "schema" => {
            let a = require_str(&params, "action")?;
            action_schema(ACTIONS, a)
        }
        other => {
            if !ACTIONS.iter().any(|a| a.name == other) {
                return Err(ToolError::UnknownAction {
                    message: format!("unknown action `{other}` for service `stash`"),
                    valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
                    hint: None,
                });
            }
            let root = require_stash_root()?;
            let store = StashStore::new(root.clone());
            store.ensure_dirs().map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".into(),
                message: format!("stash store init: {e}"),
            })?;
            dispatch_with_store(&store, other, params).await
        }
    }
}

/// Dispatch an action against a pre-constructed store.
///
/// Called by `dispatch()` after store setup, and may be called directly by
/// API handlers that hold the store in `AppState`.
pub async fn dispatch_with_store(
    store: &StashStore,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "components.list" => service::components_list(store),
        "component.get" => {
            let p = parse_get_params(&params)?;
            service::component_get(store, p)
        }
        "component.create" => {
            let p = parse_create_params(&params)?;
            service::component_create(store, p)
        }
        "component.import" => {
            let p = parse_import_params(&params)?;
            service::component_import(store, p).await
        }
        "component.workspace" => {
            let p = parse_workspace_params(&params)?;
            service::component_workspace(store, p)
        }
        "component.save" => {
            let p = parse_save_params(&params)?;
            service::component_save(store, p).await
        }
        "component.revisions" => {
            let p = parse_revisions_params(&params)?;
            service::component_revisions(store, p)
        }
        "component.export" => {
            let p = parse_export_params(&params)?;
            service::component_export(store, p).await
        }
        "component.deploy" => {
            let p = parse_deploy_params(&params)?;
            service::component_deploy(store, p).await
        }
        "providers.list" => service::providers_list(store, &params),
        "provider.link" => {
            let p = parse_link_params(&params)?;
            service::provider_link(store, p)
        }
        "provider.push" => {
            let p = parse_provider_sync_params(&params)?;
            service::provider_push(store, p)
        }
        "provider.pull" => {
            let p = parse_provider_sync_params(&params)?;
            service::provider_pull(store, p)
        }
        "targets.list" => service::targets_list(store),
        "target.add" => {
            let p = parse_target_add_params(&params)?;
            service::target_add(store, p)
        }
        "target.remove" => {
            let p = parse_target_remove_params(&params)?;
            service::target_remove(store, p)
        }
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action `{unknown}` for service `stash`"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}
