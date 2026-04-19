//! Authorization gates for the deploy service.
//!
//! Deploy requires a dedicated token separate from the general MCP bearer:
//! `LAB_DEPLOY_TOKEN` must be set in the environment before any plan/run/
//! rollback action is accepted.
//!
//! Destructive deploy actions (`run`, `rollback`) additionally require live
//! MCP elicitation when called over MCP. A client that does not advertise
//! elicitation cannot bypass the confirmation by simply setting
//! `params.confirm = true` — we refuse it at this layer.

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;
use serde_json::Value;

tokio::task_local! {
    /// Per-task MCP context set by the surface adapter before calling
    /// `dispatch_with_runner`. The CLI path scopes this to
    /// `McpContext::Cli`; the MCP adapter uses `McpElicited` or
    /// `HeadlessNoElicitation` depending on capability negotiation.
    pub static MCP_CONTEXT: McpContext;
}

/// Where the request came from, for purposes of destructive-action gating.
#[derive(Debug, Clone, Copy)]
pub enum McpContext {
    /// Command invoked from the CLI; operator confirmed via `-y`.
    Cli,
    /// HTTP call carrying a bearer token; treated like CLI for now (V1).
    // Not yet constructed: wired in when the HTTP surface adds bearer-scoped deploy dispatch.
    #[allow(dead_code)]
    HttpWithToken,
    /// MCP call whose client completed an elicitation exchange.
    McpElicited,
    /// MCP call whose client did not offer elicitation; `confirm: true`
    /// alone is insufficient.
    // Not yet constructed: wired in when the MCP surface implements elicitation negotiation.
    #[allow(dead_code)]
    HeadlessNoElicitation,
}

/// Verify `LAB_DEPLOY_TOKEN` is set and non-empty.
///
/// This is the first check every deploy action runs. The MCP HTTP bearer is
/// insufficient — deploy requires a dedicated token that the operator opts
/// into explicitly.
pub fn require_deploy_token() -> Result<(), ToolError> {
    match env_non_empty("LAB_DEPLOY_TOKEN") {
        Some(ref v) if !v.trim().is_empty() => Ok(()),
        _ => Err(ToolError::Sdk {
            sdk_kind: "auth_failed".into(),
            message: "LAB_DEPLOY_TOKEN is required for deploy actions".into(),
        }),
    }
}

/// Reject the `confirm: true` headless-bypass for destructive deploy actions.
///
/// When the caller supplies `confirm: true` but the MCP client did not
/// complete an elicitation exchange (i.e., context is
/// `HeadlessNoElicitation`), the request is refused. CLI and elicited MCP
/// calls pass through.
pub fn reject_headless_bypass(params: &Value, ctx: McpContext) -> Result<(), ToolError> {
    let confirm_present = params
        .get("confirm")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if confirm_present && matches!(ctx, McpContext::HeadlessNoElicitation) {
        return Err(ToolError::Sdk {
            sdk_kind: "auth_failed".into(),
            message: "deploy destructive actions require live MCP elicitation; \
                      `confirm: true` without an elicitation response is rejected"
                .into(),
        });
    }
    Ok(())
}

/// Read the current MCP context, falling back to `Cli` when the task-local
/// has not been scoped.
pub fn current_context() -> McpContext {
    MCP_CONTEXT.try_with(|c| *c).unwrap_or(McpContext::Cli)
}
