//! Action routing for the deploy service.

use std::future::Future;
use std::pin::Pin;

use super::authz;
use super::catalog::ACTIONS;
use super::params;
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

/// MCP-specific entry point: sync fn returning a `'static` boxed future so
/// the `dispatch_fn!` macro's `Box::pin` wrapper encloses a future with no
/// lifetime-parameterised captures (Rust issue #100013).
///
/// All synchronous work (auth, param parsing, bypass check) runs before the
/// returned future is created. The `async move` blocks capture only owned
/// values and `runner: &'static DefaultRunner`.
pub fn dispatch_mcp(
    action: String,
    params_v: Value,
    runner: &'static super::runner::DefaultRunner,
) -> Pin<Box<dyn Future<Output = Result<Value, ToolError>> + Send + 'static>> {
    match action.as_str() {
        "help" => {
            let v = help_payload("deploy", ACTIONS);
            Box::pin(async move { Ok(v) })
        }
        "schema" => {
            let result = require_str(&params_v, "action").and_then(|a| action_schema(ACTIONS, a));
            Box::pin(async move { result })
        }
        "config.list" => {
            let result = authz::require_deploy_token().and_then(|_| runner.config_list_impl());
            Box::pin(async move { result })
        }
        "plan" => {
            let auth = authz::require_deploy_token();
            let req = auth.and_then(|_| params::parse_run(&params_v).map_err(ToolError::from));
            Box::pin(async move {
                let req = req?;
                to_json(runner.plan_impl(req).await?)
            })
        }
        "run" => {
            let auth = authz::require_deploy_token();
            let req = auth.and_then(|_| params::parse_run(&params_v).map_err(ToolError::from));
            let bypass = req
                .as_ref()
                .ok()
                .map(|_| authz::reject_headless_bypass(&params_v, authz::current_context()));
            Box::pin(async move {
                let req = req?;
                if let Some(r) = bypass {
                    r?;
                }
                if !req.confirm {
                    return Err(DeployError::ValidationFailed {
                        field: "confirm".into(),
                        reason: "destructive deploy.run requires confirm=true".into(),
                    }
                    .into());
                }
                to_json(runner.run_impl(req).await?)
            })
        }
        "rollback" => {
            let auth = authz::require_deploy_token();
            let req = auth.and_then(|_| params::parse_run(&params_v).map_err(ToolError::from));
            let bypass = req
                .as_ref()
                .ok()
                .map(|_| authz::reject_headless_bypass(&params_v, authz::current_context()));
            Box::pin(async move {
                let req = req?;
                if let Some(r) = bypass {
                    r?;
                }
                if !req.confirm {
                    return Err(DeployError::ValidationFailed {
                        field: "confirm".into(),
                        reason: "destructive deploy.rollback requires confirm=true".into(),
                    }
                    .into());
                }
                to_json(runner.rollback_impl(req).await?)
            })
        }
        other => {
            let err = Err(ToolError::UnknownAction {
                message: format!("unknown action `{other}` for service `deploy`"),
                valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
                hint: None,
            });
            Box::pin(async move { err })
        }
    }
}

/// Dispatch against a concrete `DefaultRunner`. This is the entry point the
/// CLI and MCP surfaces go through once startup has built the runner from
/// config.
pub async fn dispatch_with_runner(
    action: &str,
    params_v: Value,
    runner: &super::runner::DefaultRunner,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("deploy", ACTIONS)),
        "schema" => {
            let a = require_str(&params_v, "action")?;
            action_schema(ACTIONS, a)
        }
        "config.list" => {
            authz::require_deploy_token()?;
            runner.config_list_impl()
        }
        "plan" => {
            authz::require_deploy_token()?;
            let req = params::parse_run(&params_v).map_err(ToolError::from)?;
            to_json(runner.plan_impl(req).await?)
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
            to_json(runner.run_impl(req).await?)
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
            to_json(runner.rollback_impl(req).await?)
        }
        other => Err(ToolError::UnknownAction {
            message: format!("unknown action `{other}` for service `deploy`"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}
