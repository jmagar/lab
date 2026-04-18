//! Action routing for the deploy service.

use super::authz;
use super::catalog::ACTIONS;
use super::params;
use super::runner::DeployRunner;
use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};
use lab_apis::deploy::DeployError;
use serde_json::Value;

/// Top-level dispatch without an attached runner — handles `help` / `schema`
/// and rejects any real action because orchestration requires runtime state
/// that must be injected by the caller (see `dispatch_with_runner`).
pub async fn dispatch(action: &str, params_v: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("deploy", ACTIONS)),
        "schema" => {
            let a = require_str(&params_v, "action")?;
            action_schema(ACTIONS, a)
        }
        other => {
            if !ACTIONS.iter().any(|a| a.name == other) {
                return Err(ToolError::UnknownAction {
                    message: format!("unknown action `{other}` for service `deploy`"),
                    valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
                    hint: None,
                });
            }
            authz::require_deploy_token()?;
            if other == "config.list" {
                // Without a runner we have nothing to enumerate; surface a
                // clear internal_error rather than fabricating an empty list.
                return Err(ToolError::internal_message(
                    "deploy runner is not wired into this dispatch entry point",
                ));
            }
            let req = params::parse_run(&params_v).map_err(ToolError::from)?;
            let _ = authz::reject_headless_bypass(&params_v, authz::current_context())?;
            Err(ToolError::internal_message(format!(
                "deploy runner is not wired; parsed {} target(s)",
                req.targets.len()
            )))
        }
    }
}

/// Dispatch against a concrete `DeployRunner`. This is the entry point the
/// CLI and MCP surfaces go through once startup has built the runner from
/// config.
pub async fn dispatch_with_runner<R: DeployRunner>(
    action: &str,
    params_v: Value,
    runner: &R,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("deploy", ACTIONS)),
        "schema" => {
            let a = require_str(&params_v, "action")?;
            action_schema(ACTIONS, a)
        }
        "config.list" => {
            authz::require_deploy_token()?;
            runner.config_list().await
        }
        "plan" => {
            authz::require_deploy_token()?;
            let req = params::parse_run(&params_v).map_err(ToolError::from)?;
            to_json(runner.plan(&req).await?)
        }
        "run" => {
            authz::require_deploy_token()?;
            let req = params::parse_run(&params_v).map_err(ToolError::from)?;
            authz::reject_headless_bypass(&params_v, authz::current_context())?;
            if !req.confirm {
                return Err(DeployError::ValidationFailed {
                    field: "confirm".into(),
                    reason: "destructive deploy.run requires confirm=true".into(),
                }
                .into());
            }
            to_json(runner.run(&req).await?)
        }
        "rollback" => {
            authz::require_deploy_token()?;
            let req = params::parse_run(&params_v).map_err(ToolError::from)?;
            authz::reject_headless_bypass(&params_v, authz::current_context())?;
            if !req.confirm {
                return Err(DeployError::ValidationFailed {
                    field: "confirm".into(),
                    reason: "destructive deploy.rollback requires confirm=true".into(),
                }
                .into());
            }
            to_json(runner.rollback(&req).await?)
        }
        other => Err(ToolError::UnknownAction {
            message: format!("unknown action `{other}` for service `deploy`"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}
