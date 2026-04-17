use std::convert::Infallible;

use axum::{
    Json, Router,
    extract::State,
    http::HeaderMap,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use futures::stream;
use serde_json::Value;

use crate::api::services::helpers::handle_action;
use crate::api::{ActionRequest, state::AppState};
use crate::dispatch::error::ToolError;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(handle))
        .route("/stream", get(stream_logs))
}

async fn handle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ActionRequest>,
) -> Result<Json<Value>, ToolError> {
    let request_id = headers.get("x-request-id").and_then(|v| v.to_str().ok());
    let logs_system = state.logs_system.clone().ok_or_else(|| {
        ToolError::internal_message("local log system is not wired into API state")
    })?;
    handle_action(
        "logs",
        "api",
        request_id,
        req,
        crate::dispatch::logs::ACTIONS,
        move |action, params| async move {
            crate::dispatch::logs::dispatch::dispatch_with_system(&logs_system, &action, params)
                .await
        },
    )
    .await
}

async fn stream_logs(
    State(state): State<AppState>,
) -> Result<Sse<impl futures::Stream<Item = Result<Event, Infallible>>>, ToolError> {
    let logs_system = state.logs_system.clone().ok_or_else(|| {
        ToolError::internal_message("local log system is not wired into API state")
    })?;
    let receiver = logs_system
        .subscribe(crate::dispatch::logs::types::StreamSubscription::default())
        .await?;

    let stream = stream::unfold(receiver, |mut receiver| async move {
        loop {
            match receiver.recv().await {
                Ok(event) => {
                    let payload = serde_json::to_string(&event).ok()?;
                    return Some((Ok(Event::default().data(payload)), receiver));
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
            }
        }
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
