//! HTTP route group for the `fs` workspace filesystem browser service.
//!
//! Exposes:
//! - `GET /v1/fs/list`    — directory enumeration (delegated to MCP-parallel dispatch)
//! - `GET /v1/fs/preview` — capped byte streaming from a workspace file.
//!   HTTP-only; the MCP surface refuses this action (see
//!   `crate::mcp::services::fs` for rationale).
//!
//! The `/v1` subtree already sits behind `authenticate_request` (see
//! `api/router.rs::build_router`), so no per-route auth wiring is needed.

use axum::{
    Json, Router,
    body::Body,
    extract::{Query, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Deserialize;
use serde_json::Value;
use tokio::io::AsyncReadExt;
use tokio_util::io::ReaderStream;

use crate::api::state::AppState;
use crate::dispatch::error::ToolError;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/list", get(handle_list))
        .route("/preview", get(handle_preview))
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    /// Workspace-relative path. Empty or omitted = workspace root.
    pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PreviewQuery {
    /// Workspace-relative file path. Required.
    pub path: String,
    /// Caller-suggested byte cap; server cap of 2 MiB always wins.
    pub max_bytes: Option<u64>,
}

async fn handle_list(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Value>, ToolError> {
    let root = state
        .workspace_root
        .as_ref()
        .ok_or_else(crate::dispatch::fs::not_configured_error)?;

    let params = match query.path {
        Some(p) => serde_json::json!({ "path": p }),
        None => serde_json::json!({}),
    };

    let start = std::time::Instant::now();
    let result = crate::dispatch::fs::dispatch_with_root(root.as_path(), "fs.list", params).await;
    let elapsed_ms = start.elapsed().as_millis();
    log_dispatch("fs.list", elapsed_ms, &result);
    result.map(Json)
}

/// `GET /v1/fs/preview` handler.
///
/// Streams file bytes with:
/// - `Content-Type` from the safe-MIME whitelist or `application/octet-stream`.
/// - `Content-Disposition: attachment; filename="…"` for non-inline MIMEs
///   (SVG, HTML, scripts, unknown).
/// - `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`,
///   `Content-Security-Policy: default-src 'none'; sandbox`.
///
/// The byte cap is `min(caller_max_bytes, 2 MiB)` — enforced via
/// `AsyncReadExt::take`, not `read_to_end` (which would allocate the whole
/// file before streaming).
async fn handle_preview(
    State(state): State<AppState>,
    Query(query): Query<PreviewQuery>,
) -> Result<Response, ToolError> {
    let root = state
        .workspace_root
        .as_ref()
        .ok_or_else(crate::dispatch::fs::not_configured_error)?;

    let mut params = serde_json::Map::new();
    params.insert("path".into(), Value::String(query.path.clone()));
    if let Some(n) = query.max_bytes {
        params.insert("max_bytes".into(), Value::Number(n.into()));
    }
    let params = Value::Object(params);

    let start = std::time::Instant::now();
    let preview_result = crate::dispatch::fs::open_for_preview(root.as_path(), params).await;
    let elapsed_ms = start.elapsed().as_millis();
    log_dispatch_preview(&query.path, elapsed_ms, &preview_result);
    let preview = preview_result?;

    // `take(max_bytes)` applied at the AsyncRead layer so we never buffer
    // the full file. ReaderStream chunks it and `Body::from_stream` hands
    // that to hyper.
    let limited = preview.file.take(preview.max_bytes);
    let stream = ReaderStream::new(limited);
    let body = Body::from_stream(stream);

    let disposition = if preview.inline {
        format!("inline; filename=\"{}\"", preview.disposition_filename)
    } else {
        format!("attachment; filename=\"{}\"", preview.disposition_filename)
    };

    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, preview.content_type)
        .header(header::CONTENT_DISPOSITION, disposition)
        .header("X-Content-Type-Options", "nosniff")
        .header("X-Frame-Options", "DENY")
        .header(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'none'; sandbox"),
        )
        .header(header::CACHE_CONTROL, "private, no-store")
        .body(body)
        .map_err(|e| ToolError::Sdk {
            sdk_kind: "internal_error".into(),
            message: format!("response build failed: {e}"),
        })?;

    // Explicitly clear any upstream content-length — the stream is
    // capped but unknown-length at construction time.
    response.headers_mut().remove(header::CONTENT_LENGTH);
    Ok(response)
}

fn log_dispatch(action: &'static str, elapsed_ms: u128, result: &Result<Value, ToolError>) {
    match result {
        Ok(_) => tracing::info!(
            surface = "api",
            service = "fs",
            action,
            elapsed_ms,
            "dispatch ok"
        ),
        Err(err) => {
            if err.is_internal() {
                tracing::error!(
                    surface = "api",
                    service = "fs",
                    action,
                    elapsed_ms,
                    kind = err.kind(),
                    "dispatch error"
                );
            } else {
                tracing::warn!(
                    surface = "api",
                    service = "fs",
                    action,
                    elapsed_ms,
                    kind = err.kind(),
                    "dispatch error"
                );
            }
        }
    }
}

fn log_dispatch_preview(
    path: &str,
    elapsed_ms: u128,
    result: &Result<crate::dispatch::fs::Preview, ToolError>,
) {
    match result {
        Ok(p) => tracing::info!(
            surface = "api",
            service = "fs",
            action = "fs.preview",
            elapsed_ms,
            path = %p.rel_path,
            mime = p.content_type,
            inline = p.inline,
            max_bytes = p.max_bytes,
            "dispatch ok"
        ),
        Err(err) => {
            if err.is_internal() {
                tracing::error!(
                    surface = "api",
                    service = "fs",
                    action = "fs.preview",
                    elapsed_ms,
                    kind = err.kind(),
                    path = %path,
                    "dispatch error"
                );
            } else {
                tracing::warn!(
                    surface = "api",
                    service = "fs",
                    action = "fs.preview",
                    elapsed_ms,
                    kind = err.kind(),
                    path = %path,
                    "dispatch error"
                );
            }
        }
    }
}
