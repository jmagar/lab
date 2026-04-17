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
use tracing::{error, info, warn};

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
    headers: HeaderMap,
) -> Result<Sse<impl futures::Stream<Item = Result<Event, Infallible>>>, ToolError> {
    const ACTION: &str = "logs.stream";
    let request_id = headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let start = std::time::Instant::now();

    info!(
        surface = "api",
        service = "logs",
        action = ACTION,
        request_id = request_id.as_deref(),
        "dispatch start"
    );

    let Some(logs_system) = state.logs_system.clone() else {
        let error = ToolError::internal_message("local log system is not wired into API state");
        error!(
            surface = "api",
            service = "logs",
            action = ACTION,
            request_id = request_id.as_deref(),
            elapsed_ms = start.elapsed().as_millis(),
            kind = error.kind(),
            "dispatch error"
        );
        return Err(error);
    };
    let receiver = match logs_system
        .subscribe(crate::dispatch::logs::types::StreamSubscription::default())
        .await
    {
        Ok(receiver) => receiver,
        Err(error) => {
            if error.is_internal() {
                error!(
                    surface = "api",
                    service = "logs",
                    action = ACTION,
                    request_id = request_id.as_deref(),
                    elapsed_ms = start.elapsed().as_millis(),
                    kind = error.kind(),
                    "dispatch error"
                );
            } else {
                warn!(
                    surface = "api",
                    service = "logs",
                    action = ACTION,
                    request_id = request_id.as_deref(),
                    elapsed_ms = start.elapsed().as_millis(),
                    kind = error.kind(),
                    "dispatch error"
                );
            }
            return Err(error);
        }
    };

    info!(
        surface = "api",
        service = "logs",
        action = ACTION,
        request_id = request_id.as_deref(),
        elapsed_ms = start.elapsed().as_millis(),
        "dispatch ok"
    );

    let opened_at = std::time::Instant::now();
    let request_id_for_stream = request_id.clone();

    let stream = stream::unfold(receiver, move |mut receiver| {
        let request_id = request_id_for_stream.clone();
        async move {
            loop {
                match receiver.recv().await {
                    Ok(event) => match serde_json::to_string(&event) {
                        Ok(payload) => {
                            return Some((Ok(Event::default().data(payload)), receiver));
                        }
                        Err(error) => {
                            warn!(error = %error, "failed to serialize log event for SSE; skipping");
                        }
                    },
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!(skipped, "SSE log subscriber lagged; dropping events");
                        return Some((
                            Ok(Event::default().event("lag").data(skipped.to_string())),
                            receiver,
                        ));
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        info!(
                            surface = "api",
                            service = "logs",
                            action = ACTION,
                            request_id = request_id.as_deref(),
                            elapsed_ms = opened_at.elapsed().as_millis(),
                            "dispatch finish"
                        );
                        return None;
                    }
                }
            }
        }
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
