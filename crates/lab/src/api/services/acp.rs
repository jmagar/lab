use std::convert::Infallible;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{get, post},
};
use futures::StreamExt;
use serde::Deserialize;
use serde_json::json;

use crate::api::state::AppState;
use crate::dispatch::acp::dispatch::dispatch_with_registry;
use crate::dispatch::acp::dispatch::validate_subscribe_ticket;
use crate::dispatch::error::ToolError;

/// Hard cap on incoming prompt text (64 000 chars ≈ 16 000 tokens at 4 chars/token).
const PROMPT_MAX_CHARS: usize = 64_000;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/provider", get(provider_health))
        .route("/sessions", get(list_sessions).post(create_session))
        .route("/sessions/{session_id}/prompt", post(prompt_session))
        .route("/sessions/{session_id}/cancel", post(cancel_session))
        .route("/sessions/{session_id}/events", get(stream_events))
}

async fn provider_health(State(state): State<AppState>) -> impl IntoResponse {
    match dispatch_with_registry(&state.acp_registry, "provider.list", json!({})).await {
        Ok(v) => Json(v).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn list_sessions(State(state): State<AppState>) -> impl IntoResponse {
    // TODO(phase-2): extract bearer principal from request extensions and pass it here
    // so each caller only sees their own sessions. Until bearer auth is wired in the
    // middleware layer this returns all sessions (anonymous == empty principal).
    match dispatch_with_registry(&state.acp_registry, "session.list", json!({})).await {
        Ok(v) => Json(v).into_response(),
        Err(e) => e.into_response(),
    }
}

#[derive(Deserialize)]
struct CreateSessionBody {
    provider: Option<String>,
    cwd: Option<String>,
    title: Option<String>,
}

async fn create_session(
    State(state): State<AppState>,
    Json(body): Json<CreateSessionBody>,
) -> impl IntoResponse {
    let params = json!({
        "provider": body.provider,
        "cwd": body.cwd,
        "title": body.title,
    });
    match dispatch_with_registry(&state.acp_registry, "session.start", params).await {
        Ok(v) => Json(v).into_response(),
        Err(e) => e.into_response(),
    }
}

/// Optional structured page context from the frontend.
/// All validation and injection logic lives in `dispatch/acp/page_context.rs`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PageContextBody {
    route: String,
    entity_type: Option<String>,
    entity_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PromptBody {
    prompt: String,
    /// Optional structured page context. Passed to dispatch; injection is handled there.
    page_context: Option<PageContextBody>,
}

async fn prompt_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(body): Json<PromptBody>,
) -> impl IntoResponse {
    if body.prompt.trim().is_empty() {
        return ToolError::MissingParam {
            message: "prompt is required".to_string(),
            param: "prompt".to_string(),
        }
        .into_response();
    }

    if body.prompt.len() > PROMPT_MAX_CHARS {
        return ToolError::InvalidParam {
            message: format!(
                "prompt exceeds maximum allowed length ({} > {} chars)",
                body.prompt.len(),
                PROMPT_MAX_CHARS
            ),
            param: "prompt".to_string(),
        }
        .into_response();
    }

    // Pass page_context as a JSON object to the dispatch layer.
    // All sanitization and prefix assembly happens there — HTTP handler is a thin shim.
    let page_context_value = body.page_context.as_ref().map(|ctx| {
        json!({
            "route": ctx.route,
            "entityType": ctx.entity_type,
            "entityId": ctx.entity_id,
        })
    });

    let params = json!({
        "session_id": session_id,
        "text": body.prompt.trim(),
        "page_context": page_context_value,
    });
    match dispatch_with_registry(&state.acp_registry, "session.prompt", params).await {
        Ok(v) => Json(v).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn cancel_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let params = json!({
        "session_id": session_id,
        "confirm": true,
    });
    match dispatch_with_registry(&state.acp_registry, "session.cancel", params).await {
        Ok(v) => Json(v).into_response(),
        Err(e) => e.into_response(),
    }
}

#[derive(Deserialize)]
struct EventQuery {
    since: Option<u64>,
    ticket: Option<String>,
}

async fn stream_events(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Query(query): Query<EventQuery>,
) -> Response {
    // Validate SSE ticket before establishing stream.
    let principal = if let Some(ref ticket) = query.ticket {
        match validate_subscribe_ticket(ticket) {
            Ok((tid, principal)) => {
                if tid != session_id {
                    return (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({ "kind": "auth_failed", "message": "ticket session_id mismatch" })),
                    )
                        .into_response();
                }
                principal
            }
            Err(e) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::to_value(&e).unwrap_or_default()),
                )
                    .into_response();
            }
        }
    } else {
        // No ticket provided — reject immediately (Phase 2 will wire full auth;
        // until then, every SSE caller must obtain a ticket via session.subscribe_ticket).
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "kind": "auth_failed", "message": "SSE ticket required; call session.subscribe_ticket first" })),
        )
            .into_response();
    };

    let since = query.since.unwrap_or(0);
    let stream_result = state
        .acp_registry
        .subscribe(&session_id, since, &principal)
        .await;

    match stream_result {
        Err(e) => e.into_response(),
        Ok(event_stream) => {
            let sse_stream = event_stream.map(|event| {
                let data = serde_json::to_string(&*event).unwrap_or_else(|_| "{}".to_string());
                Ok::<Event, Infallible>(Event::default().id(event.seq().to_string()).data(data))
            });
            Sse::new(sse_stream)
                .keep_alive(KeepAlive::default())
                .into_response()
        }
    }
}
