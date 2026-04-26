//! Top-level action router for the `stash` dispatch service.
//!
//! `dispatch()` handles the two built-in meta-actions (`help`, `schema`) and
//! delegates all service-specific actions to `dispatch_with_root()`.
//!
//! NOTE: Service-specific arms are stubbed with `todo!()` — full implementation
//! is Wave 7 (store.rs + I/O). Callers that reach any service arm will panic
//! until that wave lands.
//!
//! NOTE: `component.export` is marked `destructive: true` in the catalog but
//! the runtime MUST additionally check `include_secrets` from params and only
//! require confirmation when it is `true`. The `todo!()` stub below is the
//! placeholder for that conditional logic (Wave 7).

use std::path::PathBuf;

use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str};

use super::catalog::ACTIONS;
use super::client::require_stash_root;

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
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
            dispatch_with_root(root, other, params).await
        }
    }
}

pub async fn dispatch_with_root(
    root: &PathBuf,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    let _ = (root, &params); // suppress unused warnings on stub arms
    match action {
        "components.list" => todo!("stash components.list — Wave 7"),
        "component.get" => todo!("stash component.get — Wave 7"),
        "component.create" => todo!("stash component.create — Wave 7"),
        "component.import" => todo!("stash component.import — Wave 7"),
        "component.workspace" => todo!("stash component.workspace — Wave 7"),
        "component.save" => todo!("stash component.save — Wave 7"),
        "component.revisions" => todo!("stash component.revisions — Wave 7"),
        // TODO(Wave-7): component.export must check `include_secrets` param;
        //   only require MCP confirmation when include_secrets == true.
        "component.export" => todo!("stash component.export — Wave 7"),
        "component.deploy" => todo!("stash component.deploy — Wave 7"),
        "providers.list" => todo!("stash providers.list — Wave 7"),
        "provider.link" => todo!("stash provider.link — Wave 7"),
        "provider.push" => todo!("stash provider.push — Wave 7"),
        "provider.pull" => todo!("stash provider.pull — Wave 7"),
        "targets.list" => todo!("stash targets.list — Wave 7"),
        "target.add" => todo!("stash target.add — Wave 7"),
        "target.remove" => todo!("stash target.remove — Wave 7"),
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action `{unknown}` for service `stash`"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}
