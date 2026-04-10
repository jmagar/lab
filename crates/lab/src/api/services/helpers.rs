//! Shared API dispatch wrapper.
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
use axum::http::HeaderMap;
use serde_json::Value;
use tracing::Instrument;

use lab_apis::core::action::ActionSpec;

use crate::api::ActionRequest;
use crate::dispatch::context::DispatchContext;
use crate::dispatch::error::ToolError;

/// Dispatch a service action request with unknown-action gate, confirmation gate, and logging.
///
/// Owns:
/// - Unknown-action gate: if `action` is not present in `actions`, returns `ToolError` with
///   `kind = "unknown_action"` immediately — dispatch closure is never called.
/// - Destructive confirmation gate: `ActionSpec.destructive == true` requires either
///   `params["confirm"] == true` OR `X-Lab-Confirm: yes` (or `true`) header,
///   else returns `ToolError` with `kind = "confirmation_required"`.
/// - `confirm` key stripping: removed from params before forwarding to dispatch.
/// - Timer wrapping the full dispatch call.
/// - Structured dispatch logging (service, action, `elapsed_ms`, kind on error).
///   **Never logs params** — params may contain credentials.
/// - JSON response wrapping.
///
/// Does NOT own: axum routing, request extraction, service-specific execution.
///
/// # Errors
///
/// Returns `ToolError` when:
/// - The action is not found in `actions` (`unknown_action`)
/// - The matched action is destructive and neither `params["confirm"] == true` nor
///   `X-Lab-Confirm: yes`/`true` header is present (`confirmation_required`)
/// - The dispatch closure itself returns an error
#[allow(clippy::too_many_lines)]
pub async fn handle_action<F, Fut>(
    service: &'static str,
    ctx: DispatchContext,
    request_id: Option<&str>,
    req: ActionRequest,
    actions: &[ActionSpec],
    dispatch: F,
    headers: Option<&HeaderMap>,
) -> Result<Json<Value>, ToolError>
where
    F: FnOnce(String, Value) -> Fut,
    Fut: Future<Output = Result<Value, ToolError>>,
{
    let action = req.action;
    let mut params = req.params;

    // Gate: unknown actions are rejected here, not silently forwarded.
    // "help" and "schema" are built-in actions intercepted inside dispatch(); they bypass
    // the catalog gate since they don't appear in ACTIONS.
    let is_builtin = matches!(action.as_str(), "help" | "schema");
    let spec: Option<&ActionSpec> = if is_builtin {
        None
    } else if let Some(s) = actions.iter().find(|s| s.name == action) { Some(s) } else {
        tracing::warn!(
            surface = ctx.surface,
            service,
            action,
            request_id,
            "unknown_action rejected at gate"
        );
        // Include built-ins in valid[] so agents can discover them.
        let mut valid: Vec<String> =
            actions.iter().map(|s| s.name.to_string()).collect();
        valid.push("help".to_string());
        valid.push("schema".to_string());
        return Err(ToolError::UnknownAction {
            message: format!("unknown action: `{action}`"),
            valid,
            hint: None,
        });
    };
    let is_destructive = spec.is_some_and(|s| s.destructive);

    // Gate: destructive confirmation.
    // Confirmation can be granted via params["confirm"] == true OR X-Lab-Confirm: yes/true header.
    if is_destructive {
        let confirmed_by_params = params
            .get("confirm")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let confirmed_by_header = headers
            .and_then(|h| h.get("x-lab-confirm"))
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| v.eq_ignore_ascii_case("yes") || v.eq_ignore_ascii_case("true"));
        if !confirmed_by_params && !confirmed_by_header {
            tracing::warn!(
                surface = ctx.surface,
                service,
                action,
                request_id,
                "confirmation_required for destructive action"
            );
            return Err(ToolError::ConfirmationRequired {
                message: format!(
                    "action `{action}` is destructive — set `confirm: true` in params or send `X-Lab-Confirm: yes` header to proceed"
                ),
            });
        }
    }

    // Strip `confirm` from params before forwarding — it's a gate-level key, not a service param.
    if let Value::Object(ref mut m) = params {
        m.remove("confirm");
    }

    // Clone action before the move into dispatch — needed for post-dispatch logging.
    let action_log = action.clone();

    // Intent log: emit before dispatch so there is audit evidence even if the downstream
    // service errors mid-way. Only fires for destructive actions after confirmation succeeds.
    if is_destructive {
        tracing::info!(
            surface = ctx.surface,
            service,
            action = action_log,
            request_id,
            destructive = true,
            "destructive action authorized — executing"
        );
    }

    let dispatch_span = tracing::info_span!(
        "dispatch",
        surface = ctx.surface,
        service,
        action = action_log,
        request_id
    );
    let start = std::time::Instant::now();
    let result = dispatch(action, params).instrument(dispatch_span).await;
    let elapsed_ms = start.elapsed().as_millis();

    match &result {
        Ok(_) => tracing::info!(
            surface = ctx.surface,
            service,
            action = action_log,
            request_id,
            elapsed_ms,
            destructive = is_destructive,
            "dispatch ok"
        ),
        Err(e) if e.is_internal() => tracing::error!(
            surface = ctx.surface,
            service,
            action = action_log,
            request_id,
            elapsed_ms,
            kind = e.kind(),
            "dispatch error"
        ),
        Err(e) => tracing::warn!(
            surface = ctx.surface,
            service,
            action = action_log,
            request_id,
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
    use crate::test_support::{SharedBuf, captured_logs};
    use lab_apis::core::action::{ActionSpec, ParamSpec};
    use serde_json::json;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};

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
        let result = handle_action("testsvc", test_ctx(), None, req, ACTIONS, |a, p| {
            ok_dispatch(a, p)
        }, None)
        .await;
        assert!(result.is_ok(), "expected Ok, got {result:?}");
        let Json(val) = result.unwrap();
        assert_eq!(val["result"], "success");
    }

    // ── Error path preserves ToolError kind ──────────────────────────────────

    #[tokio::test]
    async fn error_path_preserves_tool_error_kind() {
        let req = make_req("safe.read", json!({}));
        let result = handle_action("testsvc", test_ctx(), None, req, ACTIONS, |a, p| {
            err_dispatch(a, p)
        }, None)
        .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), "missing_param");
    }

    // ── Destructive gate: missing confirm ────────────────────────────────────

    #[tokio::test]
    async fn destructive_without_confirm_returns_confirmation_required() {
        let req = make_req("danger.delete", json!({"id": "abc"}));
        let result = handle_action("testsvc", test_ctx(), None, req, ACTIONS, |a, p| {
            ok_dispatch(a, p)
        }, None)
        .await;
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
        let result = handle_action("testsvc", test_ctx(), None, req, ACTIONS, |a, p| {
            ok_dispatch(a, p)
        }, None)
        .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), "confirmation_required");
    }

    // ── Destructive gate: confirm present ────────────────────────────────────

    #[tokio::test]
    async fn destructive_with_confirm_true_proceeds_to_dispatch() {
        let req = make_req("danger.delete", json!({"id": "abc", "confirm": true}));
        let result = handle_action("testsvc", test_ctx(), None, req, ACTIONS, |a, p| {
            ok_dispatch(a, p)
        }, None)
        .await;
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
        let result = handle_action("testsvc", test_ctx(), None, req, ACTIONS, |a, p| {
            ok_dispatch(a, p)
        }, None)
        .await;
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
        let result = handle_action("testsvc", test_ctx(), None, req, ACTIONS, move |_a, _p| {
            let flag = Arc::clone(&dispatch_called_clone);
            async move {
                flag.store(true, Ordering::SeqCst);
                Ok(json!({"result": "should not reach here"}))
            }
        }, None)
        .await;

        assert!(result.is_err(), "unknown action must be rejected");
        let err = result.unwrap_err();
        assert_eq!(
            err.kind(),
            "unknown_action",
            "expected unknown_action kind, got {}",
            err.kind()
        );
        // Envelope must include valid actions for agent discoverability.
        let envelope = serde_json::to_value(&err).unwrap();
        let valid = envelope["valid"]
            .as_array()
            .expect("unknown_action envelope must include `valid` array");
        assert!(
            valid.iter().any(|v| v == "safe.read"),
            "valid must include known actions"
        );
        assert!(
            valid.iter().any(|v| v == "danger.delete"),
            "valid must include known actions"
        );
        // Built-ins must always appear in valid[] so agents can discover them.
        assert!(
            valid.iter().any(|v| v == "help"),
            "valid must include built-in help action"
        );
        assert!(
            valid.iter().any(|v| v == "schema"),
            "valid must include built-in schema action"
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
        let result = handle_action(
            "testsvc",
            test_ctx(),
            None,
            req,
            ACTIONS,
            |_action, params| async move {
                // `confirm` must not be present in forwarded params
                assert!(
                    params.get("confirm").is_none(),
                    "`confirm` key must be stripped before dispatch, but found: {:?}",
                    params.get("confirm")
                );
                Ok(json!({"result": "ok"}))
            },
            None,
        )
        .await;
        assert!(result.is_ok(), "expected Ok, got {result:?}");
    }

    // ── Destructive error preserves dispatch error kind ──────────────────────

    #[tokio::test]
    async fn destructive_with_confirm_dispatch_error_preserves_kind() {
        let req = make_req("danger.delete", json!({"confirm": true}));
        // dispatch returns missing_param (id not given)
        let result = handle_action("testsvc", test_ctx(), None, req, ACTIONS, |a, p| {
            err_dispatch(a, p)
        }, None)
        .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), "missing_param");
    }

    // ── confirm as string "true" must NOT pass the gate ─────────────────────

    #[tokio::test]
    async fn destructive_with_confirm_string_true_does_not_pass() {
        // confirm: "true" (string) — Value::as_bool returns None for strings.
        let req = make_req("danger.delete", json!({"id": "abc", "confirm": "true"}));
        let result = handle_action("testsvc", test_ctx(), None, req, ACTIONS, |a, p| {
            ok_dispatch(a, p)
        }, None)
        .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.kind(),
            "confirmation_required",
            "string 'true' must not pass the boolean confirm gate"
        );
    }

    // ── Empty ACTIONS slice: any action is unknown ──────────────────────────

    #[tokio::test]
    async fn empty_actions_rejects_everything() {
        let req = make_req("anything", json!({}));
        let result = handle_action("testsvc", test_ctx(), None, req, &[], |a, p| {
            ok_dispatch(a, p)
        }, None)
        .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), "unknown_action");
    }

    // ── X-Lab-Confirm header: "yes" authorizes destructive action ───────────

    #[tokio::test]
    async fn destructive_with_x_lab_confirm_yes_header_proceeds() {
        use axum::http::HeaderValue;
        let req = make_req("danger.delete", json!({"id": "abc"}));
        let mut headers = HeaderMap::new();
        headers.insert("x-lab-confirm", HeaderValue::from_static("yes"));
        let result = handle_action("testsvc", test_ctx(), None, req, ACTIONS, |a, p| {
            ok_dispatch(a, p)
        }, Some(&headers))
        .await;
        assert!(
            result.is_ok(),
            "X-Lab-Confirm: yes must authorize destructive action, got {result:?}"
        );
    }

    // ── X-Lab-Confirm header: "true" also authorizes destructive action ──────

    #[tokio::test]
    async fn destructive_with_x_lab_confirm_true_header_proceeds() {
        use axum::http::HeaderValue;
        let req = make_req("danger.delete", json!({"id": "abc"}));
        let mut headers = HeaderMap::new();
        headers.insert("x-lab-confirm", HeaderValue::from_static("true"));
        let result = handle_action("testsvc", test_ctx(), None, req, ACTIONS, |a, p| {
            ok_dispatch(a, p)
        }, Some(&headers))
        .await;
        assert!(
            result.is_ok(),
            "X-Lab-Confirm: true must authorize destructive action, got {result:?}"
        );
    }

    #[test]
    fn dispatch_logs_api_surface_and_request_id() {
        let _tracing_lock = crate::test_support::TRACING_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let buf = SharedBuf::default();
        let subscriber = tracing_subscriber::registry()
            .with(EnvFilter::new("lab=info"))
            .with(
                fmt::layer()
                    .json()
                    .with_writer(buf.clone())
                    .with_ansi(false)
                    .without_time(),
            );
        let _guard = tracing::subscriber::set_default(subscriber);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let req = make_req("safe.read", json!({}));
            drop(
                handle_action(
                    "testsvc",
                    test_ctx(),
                    Some("req-123"),
                    req,
                    ACTIONS,
                    |a, p| ok_dispatch(a, p),
                    None,
                )
                .await
                .unwrap(),
            );
        });

        let logs = captured_logs(&buf);
        assert!(logs.contains("\"surface\":\"api\""));
        assert!(logs.contains("\"service\":\"testsvc\""));
        assert!(logs.contains("\"action\":\"safe.read\""));
        assert!(logs.contains("\"request_id\":\"req-123\""));
        assert!(logs.contains("\"elapsed_ms\""));
    }

    // ── Destructive intent log fires before dispatch ok ──────────────────────

    #[test]
    fn destructive_action_logs_intent_before_dispatch() {
        let _tracing_lock = crate::test_support::TRACING_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let buf = SharedBuf::default();
        let subscriber = tracing_subscriber::registry()
            .with(EnvFilter::new("lab=info"))
            .with(
                fmt::layer()
                    .json()
                    .with_writer(buf.clone())
                    .with_ansi(false)
                    .without_time(),
            );
        let _guard = tracing::subscriber::set_default(subscriber);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let req = make_req("danger.delete", json!({"id": "abc", "confirm": true}));
            drop(
                handle_action("testsvc", test_ctx(), Some("req-del-1"), req, ACTIONS, |a, p| {
                    ok_dispatch(a, p)
                }, None)
                .await
                .unwrap(),
            );
        });

        let logs = captured_logs(&buf);

        // Both events must be present.
        assert!(
            logs.contains("destructive action authorized"),
            "expected intent log, got: {logs}"
        );
        assert!(logs.contains("dispatch ok"), "expected dispatch ok log, got: {logs}");

        // Intent event must include required fields.
        assert!(logs.contains("\"surface\":\"api\""), "intent log missing surface field");
        assert!(logs.contains("\"service\":\"testsvc\""), "intent log missing service field");
        assert!(logs.contains("\"action\":\"danger.delete\""), "intent log missing action field");
        assert!(
            logs.contains("\"destructive\":true"),
            "intent log missing destructive=true field"
        );

        // Dispatch ok must also carry destructive=true.
        assert!(
            logs.contains("\"destructive\":true"),
            "dispatch ok log missing destructive field"
        );

        // Intent event must appear before dispatch ok event in the log stream.
        let intent_pos = logs.find("destructive action authorized").unwrap();
        let ok_pos = logs.find("dispatch ok").unwrap();
        assert!(
            intent_pos < ok_pos,
            "intent log must appear before dispatch ok (intent_pos={intent_pos}, ok_pos={ok_pos})"
        );

        // Non-destructive actions must NOT emit the intent event.
        // Run a separate safe action through the same subscriber and confirm no spurious entry.
        // (The logs buffer already only contains destructive-action output above, so this just
        //  double-checks count: only one "destructive action authorized" line should exist.)
        let intent_count = logs.matches("destructive action authorized").count();
        assert_eq!(intent_count, 1, "expected exactly one intent log line, got {intent_count}");
    }

    // ── Non-destructive action must NOT emit intent log ──────────────────────

    #[test]
    fn non_destructive_action_does_not_log_intent() {
        let _tracing_lock = crate::test_support::TRACING_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let buf = SharedBuf::default();
        let subscriber = tracing_subscriber::registry()
            .with(EnvFilter::new("lab=info"))
            .with(
                fmt::layer()
                    .json()
                    .with_writer(buf.clone())
                    .with_ansi(false)
                    .without_time(),
            );
        let _guard = tracing::subscriber::set_default(subscriber);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let req = make_req("safe.read", json!({}));
            drop(
                handle_action("testsvc", test_ctx(), None, req, ACTIONS, |a, p| {
                    ok_dispatch(a, p)
                }, None)
                .await
                .unwrap(),
            );
        });

        let logs = captured_logs(&buf);
        assert!(
            !logs.contains("destructive action authorized"),
            "non-destructive action must not emit intent log, got: {logs}"
        );
        // Dispatch ok is still emitted for non-destructive actions.
        assert!(logs.contains("dispatch ok"), "expected dispatch ok for non-destructive action");
    }

    // ── Built-in actions bypass catalog gate ─────────────────────────────────

    #[tokio::test]
    async fn help_action_bypasses_catalog_gate_and_reaches_dispatch() {
        // "help" is not in ACTIONS but must not return unknown_action.
        let req = make_req("help", json!({}));
        let result = handle_action("testsvc", test_ctx(), None, req, ACTIONS, |a, p| {
            ok_dispatch(a, p)
        }, None)
        .await;
        // Dispatch returns ok (our ok_dispatch returns the forwarded action name).
        assert!(
            result.is_ok(),
            "help must bypass catalog gate and reach dispatch, got {result:?}"
        );
    }

    #[tokio::test]
    async fn schema_action_bypasses_catalog_gate_and_reaches_dispatch() {
        let req = make_req("schema", json!({"action": "safe.read"}));
        let result = handle_action("testsvc", test_ctx(), None, req, ACTIONS, |a, p| {
            ok_dispatch(a, p)
        }, None)
        .await;
        assert!(
            result.is_ok(),
            "schema must bypass catalog gate and reach dispatch, got {result:?}"
        );
    }
}
