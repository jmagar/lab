use std::convert::Infallible;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use futures::stream;
use serde::Deserialize;
use tokio::sync::broadcast;

use crate::acp::types::{BridgeEvent, StartSessionInput};
use crate::api::state::AppState;
use crate::dispatch::error::ToolError;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/provider", get(provider_health))
        .route("/sessions", get(list_sessions).post(create_session))
        .route("/sessions/{session_id}/prompt", post(prompt_session))
        .route("/sessions/{session_id}/cancel", post(cancel_session))
        .route("/sessions/{session_id}/events", get(stream_events))
}

async fn provider_health(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "provider": state.acp_registry.provider_health()
    }))
}

async fn list_sessions(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ToolError> {
    // principal = "" until bead 7 wires auth
    let sessions = state.acp_registry.list_sessions("").await;
    Ok(Json(serde_json::json!({ "sessions": sessions })))
}

#[derive(Deserialize)]
struct CreateSessionBody {
    cwd: Option<String>,
    title: Option<String>,
}

async fn create_session(
    State(state): State<AppState>,
    Json(body): Json<CreateSessionBody>,
) -> Result<Json<serde_json::Value>, ToolError> {
    let input = StartSessionInput {
        cwd: body.cwd.unwrap_or_else(|| {
            std::env::var("ACP_SESSION_CWD").unwrap_or_else(|_| {
                std::env::current_dir()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|_| ".".to_string())
            })
        }),
        title: body.title,
        // principal = None until bead 7 wires auth
        principal: None,
    };
    // principal = "" until bead 7 wires auth
    let session = state
        .acp_registry
        .create_session(input, "")
        .await?;
    Ok(Json(serde_json::json!({ "session": session })))
}

#[derive(Deserialize)]
struct PromptBody {
    prompt: String,
}

async fn prompt_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(body): Json<PromptBody>,
) -> Result<Json<serde_json::Value>, ToolError> {
    let prompt = body.prompt.trim().to_string();
    if prompt.is_empty() {
        return Err(ToolError::MissingParam {
            message: "prompt is required".to_string(),
            param: "prompt".to_string(),
        });
    }
    // principal = "" until bead 7 wires auth
    state
        .acp_registry
        .prompt_session(&session_id, &prompt, "")
        .await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn cancel_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<serde_json::Value>, ToolError> {
    // principal = "" until bead 7 wires auth
    state.acp_registry.cancel_session(&session_id, "").await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(Deserialize)]
struct EventQuery {
    since: Option<u64>,
}

async fn stream_events(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Query(query): Query<EventQuery>,
) -> Result<Sse<impl futures::Stream<Item = Result<Event, Infallible>>>, ToolError> {
    // Use subscribe_legacy until bead 7 migrates to the new Stream<AcpEvent> shape.
    let (backlog, receiver): (Vec<BridgeEvent>, broadcast::Receiver<BridgeEvent>) = state
        .acp_registry
        .subscribe_legacy(&session_id, query.since.unwrap_or(0))
        .await?;
    let stream = stream::unfold(
        (
            backlog.into_iter(),
            receiver,
            Arc::new(session_id),
        ),
        |(
            mut backlog,
            mut receiver,
            session_id,
        ): (
            std::vec::IntoIter<BridgeEvent>,
            broadcast::Receiver<BridgeEvent>,
            Arc<String>,
        )| async move {
            if let Some(event) = backlog.next() {
                return Some((Ok(sse_event(event)), (backlog, receiver, session_id)));
            }

            match receiver.recv().await {
                Ok(event) => Some((Ok(sse_event(event)), (backlog, receiver, session_id))),
                Err(broadcast::error::RecvError::Lagged(skipped)) => Some((
                    Ok(Event::default().event("lag").data(skipped.to_string())),
                    (backlog, receiver, session_id),
                )),
                Err(broadcast::error::RecvError::Closed) => None,
            }
        },
    );

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

fn sse_event(event: BridgeEvent) -> Event {
    Event::default()
        .id(event.seq.to_string())
        .data(serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string()))
}
