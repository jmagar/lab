//! Shared HTTP API dispatch wrapper.
//!
//! `handle_action` is the single enforcement point for:
//! - Destructive-action confirmation gate (security requirement)
//! - Dispatch timing + structured logging
//! - JSON response wrapping
//!
//! IMPORTANT: Params are NEVER logged — they may contain credentials.
//! See `docs/OBSERVABILITY.md`.

use std::future::Future;

use axum::Json;
use serde_json::Value;

use lab_apis::core::action::ActionSpec;

use crate::api::ActionRequest;
use crate::mcp::envelope::ToolError;

/// Dispatch a service action request with confirmation gate and logging.
///
/// Owns:
/// - Destructive confirmation gate: `ActionSpec.destructive == true` requires
///   `params["confirm"] == true`, else returns `ToolError` with `kind = "confirmation_required"`.
/// - Timer wrapping the full dispatch call.
/// - Structured dispatch logging (service, action, elapsed_ms, kind on error).
///   **Never logs params** — params may contain credentials.
/// - JSON response wrapping.
///
/// Does NOT own: axum routing, request extraction, service-specific execution.
///
/// # Errors
///
/// Returns `ToolError` when:
/// - The matched action is destructive and `params["confirm"] != true`
/// - The dispatch closure itself returns an error
pub async fn handle_action<F, Fut>(
    service: &'static str,
    req: ActionRequest,
    actions: &[ActionSpec],
    dispatch: F,
) -> Result<Json<Value>, ToolError>
where
    F: FnOnce(String, Value) -> Fut,
    Fut: Future<Output = Result<Value, ToolError>>,
{
    let action = req.action.clone();
    let params = req.params;

    // Destructive confirmation gate.
    if let Some(spec) = actions.iter().find(|s| s.name == action) {
        if spec.destructive {
            let confirmed = params
                .get("confirm")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if !confirmed {
                tracing::warn!(
                    surface = "http",
                    service,
                    action,
                    "confirmation_required for destructive action"
                );
                return Err(ToolError::Sdk {
                    sdk_kind: "confirmation_required".into(),
                    message: format!(
                        "action `{action}` is destructive — set `confirm: true` in params to proceed"
                    ),
                });
            }
        }
    }

    // Timed dispatch — no params in log fields.
    let start = std::time::Instant::now();
    let result = dispatch(action.clone(), params).await;
    let elapsed_ms = start.elapsed().as_millis();

    match &result {
        Ok(_) => tracing::info!(
            surface = "http",
            service,
            action,
            elapsed_ms,
            "dispatch ok"
        ),
        Err(e) => tracing::warn!(
            surface = "http",
            service,
            action,
            elapsed_ms,
            kind = e.kind(),
            "dispatch error"
        ),
    }

    result.map(Json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lab_apis::core::action::{ActionSpec, ParamSpec};
    use serde_json::json;

    // ── Fixtures ─────────────────────────────────────────────────────────────

    const ACTIONS: &[ActionSpec] = &[
        ActionSpec {
            name: "safe.read",
            description: "A non-destructive read action",
            destructive: false,
            returns: "Value",
            params: &[],
        },
        ActionSpec {
            name: "danger.delete",
            description: "A destructive delete action",
            destructive: true,
            returns: "void",
            params: &[ParamSpec {
                name: "id",
                ty: "string",
                required: true,
                description: "Resource ID",
            }],
        },
    ];

    fn make_req(action: &str, params: Value) -> ActionRequest {
        ActionRequest {
            action: action.to_string(),
            params,
        }
    }

    /// Dispatch closure that always succeeds with a fixed value.
    async fn ok_dispatch(_action: String, _params: Value) -> Result<Value, ToolError> {
        Ok(json!({"result": "success"}))
    }

    /// Dispatch closure that always fails with a fixed error.
    async fn err_dispatch(_action: String, _params: Value) -> Result<Value, ToolError> {
        Err(ToolError::MissingParam {
            message: "missing required parameter `id`".into(),
            param: "id".into(),
        })
    }

    // ── Success path ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn success_path_returns_json_value() {
        let req = make_req("safe.read", json!({}));
        let result = handle_action("testsvc", req, ACTIONS, |a, p| ok_dispatch(a, p)).await;
        assert!(result.is_ok(), "expected Ok, got {result:?}");
        let Json(val) = result.unwrap();
        assert_eq!(val["result"], "success");
    }

    // ── Error path preserves ToolError kind ──────────────────────────────────

    #[tokio::test]
    async fn error_path_preserves_tool_error_kind() {
        let req = make_req("safe.read", json!({}));
        let result = handle_action("testsvc", req, ACTIONS, |a, p| err_dispatch(a, p)).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), "missing_param");
    }

    // ── Destructive gate: missing confirm ────────────────────────────────────

    #[tokio::test]
    async fn destructive_without_confirm_returns_confirmation_required() {
        let req = make_req("danger.delete", json!({"id": "abc"}));
        let result = handle_action("testsvc", req, ACTIONS, |a, p| ok_dispatch(a, p)).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.kind(),
            "confirmation_required",
            "expected confirmation_required, got {}",
            err.kind()
        );
    }

    #[tokio::test]
    async fn destructive_with_confirm_false_returns_confirmation_required() {
        let req = make_req("danger.delete", json!({"id": "abc", "confirm": false}));
        let result = handle_action("testsvc", req, ACTIONS, |a, p| ok_dispatch(a, p)).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), "confirmation_required");
    }

    // ── Destructive gate: confirm present ────────────────────────────────────

    #[tokio::test]
    async fn destructive_with_confirm_true_proceeds_to_dispatch() {
        let req = make_req("danger.delete", json!({"id": "abc", "confirm": true}));
        let result = handle_action("testsvc", req, ACTIONS, |a, p| ok_dispatch(a, p)).await;
        assert!(
            result.is_ok(),
            "expected dispatch to proceed with confirm=true, got {result:?}"
        );
    }

    // ── Non-destructive action proceeds without confirmation ─────────────────

    #[tokio::test]
    async fn non_destructive_action_proceeds_without_confirm() {
        // No "confirm" key at all — should NOT be blocked.
        let req = make_req("safe.read", json!({}));
        let result = handle_action("testsvc", req, ACTIONS, |a, p| ok_dispatch(a, p)).await;
        assert!(
            result.is_ok(),
            "non-destructive action must not require confirmation"
        );
    }

    // ── Unknown action (not in ACTIONS) falls through to dispatch ────────────

    #[tokio::test]
    async fn unknown_action_not_in_actions_passes_to_dispatch() {
        // The gate only checks specs in `actions` — unknown names fall through
        // to the dispatch closure which owns the unknown_action error.
        let req = make_req("nonexistent.action", json!({}));
        // Use ok_dispatch: gate doesn't know it's destructive (no spec), so it
        // should let it through. Actual services would return unknown_action from
        // their own dispatch closure.
        let result = handle_action("testsvc", req, ACTIONS, |a, p| ok_dispatch(a, p)).await;
        assert!(result.is_ok(), "unknown action should pass through to dispatch");
    }

    // ── Destructive error preserves dispatch error kind ──────────────────────

    #[tokio::test]
    async fn destructive_with_confirm_dispatch_error_preserves_kind() {
        let req = make_req("danger.delete", json!({"confirm": true}));
        // dispatch returns missing_param (id not given)
        let result = handle_action("testsvc", req, ACTIONS, |a, p| err_dispatch(a, p)).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), "missing_param");
    }
}
