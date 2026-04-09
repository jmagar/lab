//! Shared HTTP API dispatch wrapper.
//!
//! `handle_action` is the single enforcement point for:
//! - Unknown-action rejection gate (fail-closed — dispatch is never reached for unknown actions)
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
use crate::services::context::DispatchContext;

/// Dispatch a service action request with unknown-action gate, confirmation gate, and logging.
///
/// Owns:
/// - Unknown-action gate: if `action` is not present in `actions`, returns `ToolError` with
///   `kind = "unknown_action"` immediately — dispatch closure is never called.
/// - Destructive confirmation gate: `ActionSpec.destructive == true` requires
///   `params["confirm"] == true`, else returns `ToolError` with `kind = "confirmation_required"`.
/// - `confirm` key stripping: removed from params before forwarding to dispatch.
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
/// - The action is not found in `actions` (unknown_action)
/// - The matched action is destructive and `params["confirm"] != true` (confirmation_required)
/// - The dispatch closure itself returns an error
pub async fn handle_action<F, Fut>(
    service: &'static str,
    ctx: DispatchContext,
    req: ActionRequest,
    actions: &[ActionSpec],
    dispatch: F,
) -> Result<Json<Value>, ToolError>
where
    F: FnOnce(String, Value) -> Fut,
    Fut: Future<Output = Result<Value, ToolError>>,
{
    let action = req.action;
    let mut params = req.params;

    // Gate: unknown actions are rejected here, not silently forwarded.
    let Some(spec) = actions.iter().find(|s| s.name == action) else {
        tracing::warn!(
            surface = ctx.surface,
            service,
            action,
            "unknown_action rejected at gate"
        );
        return Err(ToolError::Sdk {
            sdk_kind: "unknown_action".into(),
            message: format!("unknown action: `{action}`"),
        });
    };

    // Gate: destructive confirmation.
    if spec.destructive {
        let confirmed = params
            .get("confirm")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if !confirmed {
            tracing::warn!(
                surface = ctx.surface,
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

    // Strip `confirm` from params before forwarding — it's a gate-level key, not a service param.
    if let Value::Object(ref mut m) = params {
        m.remove("confirm");
    }

    // Timed dispatch — no params in log fields.
    let start = std::time::Instant::now();
    let result = dispatch(action, params).await;
    let elapsed_ms = start.elapsed().as_millis();

    // Borrow action from result context via the closure's captured binding — already moved.
    // Re-borrow service/ctx for logging.
    match &result {
        Ok(_) => tracing::info!(
            surface = ctx.surface,
            service,
            elapsed_ms,
            "dispatch ok"
        ),
        Err(e) => tracing::warn!(
            surface = ctx.surface,
            service,
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
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

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

    fn test_ctx() -> DispatchContext {
        DispatchContext {
            surface: "api",
            instance: None,
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
        let result = handle_action("testsvc", test_ctx(), req, ACTIONS, |a, p| ok_dispatch(a, p)).await;
        assert!(result.is_ok(), "expected Ok, got {result:?}");
        let Json(val) = result.unwrap();
        assert_eq!(val["result"], "success");
    }

    // ── Error path preserves ToolError kind ──────────────────────────────────

    #[tokio::test]
    async fn error_path_preserves_tool_error_kind() {
        let req = make_req("safe.read", json!({}));
        let result = handle_action("testsvc", test_ctx(), req, ACTIONS, |a, p| err_dispatch(a, p)).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), "missing_param");
    }

    // ── Destructive gate: missing confirm ────────────────────────────────────

    #[tokio::test]
    async fn destructive_without_confirm_returns_confirmation_required() {
        let req = make_req("danger.delete", json!({"id": "abc"}));
        let result = handle_action("testsvc", test_ctx(), req, ACTIONS, |a, p| ok_dispatch(a, p)).await;
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
        let result = handle_action("testsvc", test_ctx(), req, ACTIONS, |a, p| ok_dispatch(a, p)).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), "confirmation_required");
    }

    // ── Destructive gate: confirm present ────────────────────────────────────

    #[tokio::test]
    async fn destructive_with_confirm_true_proceeds_to_dispatch() {
        let req = make_req("danger.delete", json!({"id": "abc", "confirm": true}));
        let result = handle_action("testsvc", test_ctx(), req, ACTIONS, |a, p| ok_dispatch(a, p)).await;
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
        let result = handle_action("testsvc", test_ctx(), req, ACTIONS, |a, p| ok_dispatch(a, p)).await;
        assert!(
            result.is_ok(),
            "non-destructive action must not require confirmation"
        );
    }

    // ── Unknown action: gate must fail-closed, dispatch must NOT be called ───

    #[tokio::test]
    async fn unknown_action_returns_unknown_action_and_dispatch_not_called() {
        let dispatch_called = Arc::new(AtomicBool::new(false));
        let dispatch_called_clone = Arc::clone(&dispatch_called);

        let req = make_req("nonexistent.action", json!({}));
        let result = handle_action("testsvc", test_ctx(), req, ACTIONS, move |_a, _p| {
            let flag = Arc::clone(&dispatch_called_clone);
            async move {
                flag.store(true, Ordering::SeqCst);
                Ok(json!({"result": "should not reach here"}))
            }
        })
        .await;

        assert!(result.is_err(), "unknown action must be rejected");
        let err = result.unwrap_err();
        assert_eq!(
            err.kind(),
            "unknown_action",
            "expected unknown_action kind, got {}",
            err.kind()
        );
        assert!(
            !dispatch_called.load(Ordering::SeqCst),
            "dispatch closure must NOT be called for unknown actions"
        );
    }

    // ── confirm is stripped from params before dispatch ───────────────────────

    #[tokio::test]
    async fn confirm_key_stripped_from_params_before_dispatch() {
        let req = make_req("danger.delete", json!({"id": "abc", "confirm": true}));
        let result = handle_action("testsvc", test_ctx(), req, ACTIONS, |_action, params| async move {
            // `confirm` must not be present in forwarded params
            assert!(
                params.get("confirm").is_none(),
                "`confirm` key must be stripped before dispatch, but found: {:?}",
                params.get("confirm")
            );
            Ok(json!({"result": "ok"}))
        })
        .await;
        assert!(result.is_ok(), "expected Ok, got {result:?}");
    }

    // ── Destructive error preserves dispatch error kind ──────────────────────

    #[tokio::test]
    async fn destructive_with_confirm_dispatch_error_preserves_kind() {
        let req = make_req("danger.delete", json!({"confirm": true}));
        // dispatch returns missing_param (id not given)
        let result = handle_action("testsvc", test_ctx(), req, ACTIONS, |a, p| err_dispatch(a, p)).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), "missing_param");
    }
}
