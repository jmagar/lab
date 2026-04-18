//! MCP adapter for the `deploy` service — thin bridge to
//! `dispatch::deploy`.
//!
//! Destructive deploy actions require live MCP elicitation. The `entry`
//! helper maps the `elicited` signal from the MCP server into the
//! `McpContext` enum read by `authz::reject_headless_bypass`.

use serde_json::Value;

use crate::dispatch::deploy;
use crate::dispatch::deploy::authz::McpContext;
use crate::dispatch::error::ToolError;

pub const ACTIONS: &[lab_apis::core::action::ActionSpec] = deploy::catalog::ACTIONS;

/// Dispatch using the default no-op runner.
///
/// Real runtime wiring (pulling the configured `DefaultRunner` from
/// `AppState`) is the follow-up in the startup task. This thin bridge
/// exists now so the MCP registry can enumerate the `deploy` tool without
/// panicking when it is invoked.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    let runner = deploy::runner::NoopRunner;
    deploy::authz::MCP_CONTEXT
        .scope(
            McpContext::HeadlessNoElicitation,
            deploy::dispatch_with_runner(action, params, &runner),
        )
        .await
}

/// Dispatch with an explicit MCP context (used by tests and by the MCP
/// server after it has negotiated elicitation capability).
#[allow(dead_code)]
pub async fn dispatch_with_context(
    action: &str,
    params: Value,
    ctx: McpContext,
) -> Result<Value, ToolError> {
    let runner = deploy::runner::NoopRunner;
    deploy::authz::MCP_CONTEXT
        .scope(ctx, deploy::dispatch_with_runner(action, params, &runner))
        .await
}
