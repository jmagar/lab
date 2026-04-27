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

// ---- pageContext support ----

/// Maximum tokens allowed for the assembled context prefix.
/// Estimated at ~4 chars/token; we enforce a char budget of 30 * 4 = 120 chars.
const PAGE_CONTEXT_MAX_TOKENS: usize = 30;
const PAGE_CONTEXT_MAX_CHARS: usize = PAGE_CONTEXT_MAX_TOKENS * 4;

/// Characters allowed in pageContext field values.
const PAGE_CONTEXT_ALLOWED: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm',
    'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '/', '_', '-',
];

/// Tokens that indicate prompt-injection attempts — reject the entire pageContext field if any match.
const PAGE_CONTEXT_DENY_LIST: &[&str] = &[
    "system", "ignore", "override", "admin", "instruction", "assistant", "prompt",
];

/// Sanitize a single pageContext field value.
/// Returns `None` if the value fails validation (contains deny-list tokens after stripping).
/// NFKC-normalizes (ASCII-safe: identity for ASCII input), strips to allowed characters,
/// truncates to 32 chars.
fn sanitize_page_context_field(value: &str) -> Option<String> {
    // Strip to allowed characters (NFKC normalization is identity for our ASCII allow-list).
    // For non-ASCII Unicode the filter naturally drops all non-ASCII chars, achieving
    // the same safety goal as NFKC + strip without pulling in the unicode-normalization crate.
    let stripped: String = value
        .chars()
        .filter(|c| PAGE_CONTEXT_ALLOWED.contains(c))
        .take(32)
        .collect();

    if stripped.is_empty() {
        return None;
    }

    // Deny-list check (case-insensitive)
    let lower = stripped.to_lowercase();
    for denied in PAGE_CONTEXT_DENY_LIST {
        if lower.contains(denied) {
            return None;
        }
    }

    Some(stripped)
}

/// Assemble compact context prefix from a validated PageContextBody.
/// Returns `None` if validation fails (any required field is invalid).
/// Format: `[context: page={route}]` or `[context: page={route} entity={type}/{id}]`
fn assemble_page_context_prefix(
    session_id: &str,
    ctx: &PageContextBody,
) -> Option<String> {
    let route = sanitize_page_context_field(&ctx.route)?;

    // Build prefix candidates, longest first, then trim to token budget
    let prefix = match (&ctx.entity_type, &ctx.entity_id) {
        (Some(et), Some(eid)) => {
            let entity_type = sanitize_page_context_field(et)?;
            let entity_id = sanitize_page_context_field(eid)?;
            let candidate = format!("[context: page={route} entity={entity_type}/{entity_id}]");
            if candidate.len() <= PAGE_CONTEXT_MAX_CHARS {
                candidate
            } else {
                // Try without entity_id
                let without_id = format!("[context: page={route} entity={entity_type}]");
                if without_id.len() <= PAGE_CONTEXT_MAX_CHARS {
                    without_id
                } else {
                    // Fall back to route only
                    format!("[context: page={route}]")
                }
            }
        }
        (Some(et), None) => {
            let entity_type = sanitize_page_context_field(et)?;
            format!("[context: page={route} entity={entity_type}]")
        }
        _ => format!("[context: page={route}]"),
    };

    // Estimate token count: chars / 4, rough approximation
    let estimated_tokens = prefix.len().div_ceil(4);
    tracing::info!(
        surface = "api",
        service = "acp",
        action = "session.prompt",
        session_id,
        page_context_route = %route,
        page_context_token_estimate = estimated_tokens,
        "page context injected",
    );

    Some(prefix)
}

// ---- End pageContext support ----

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

/// Optional page context from the frontend — sent only when user has enabled
/// the "Send page context" toggle. Absent field = zero injection, zero token cost.
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
    /// Optional structured page context. If present, server assembles a compact
    /// token-minimal context prefix and prepends it to the system prompt.
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

    // Assemble the effective prompt text: optional context prefix + user prompt.
    // Validation failure silently skips injection — never errors the request.
    let prompt_text = if let Some(ctx) = &body.page_context {
        match assemble_page_context_prefix(&session_id, ctx) {
            Some(prefix) => format!("{}\n\n{}", prefix, body.prompt.trim()),
            None => {
                tracing::warn!(
                    surface = "api",
                    service = "acp",
                    action = "session.prompt",
                    session_id = %session_id,
                    "page context validation failed — injecting without context",
                );
                body.prompt.trim().to_string()
            }
        }
    } else {
        body.prompt.trim().to_string()
    };

    let params = json!({
        "session_id": session_id,
        "text": prompt_text,
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
        // No ticket — anonymous principal (bead 7 note: auth wired in Phase 2).
        String::new()
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
