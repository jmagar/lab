//! Read-only REST handlers for the MCP server registry at `/v0.1`.
//!
//! Three GET endpoints mirror the upstream mcpregistry.io API shape:
//!   GET /v0.1/servers
//!   GET /v0.1/servers/:serverName/versions
//!   GET /v0.1/servers/:serverName/versions/:version

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use serde::Deserialize;
use serde_json::json;

use crate::api::state::AppState;
use crate::dispatch::error::ToolError;
use crate::dispatch::mcpregistry::store::StoreListParams;

/// Query params for `GET /v0.1/servers`.
///
/// Thin deserializable wrapper — `StoreListParams` is not `Deserialize`.
#[derive(Debug, Deserialize, Default)]
pub struct ListServersQuery {
    /// Substring search on server name.
    pub search: Option<String>,
    /// Opaque pagination cursor from a previous response.
    pub cursor: Option<String>,
    /// Max results per page (server-side clamped to 1–100, default 20).
    pub limit: Option<u32>,
    /// Include servers with `status = 'deleted'`.
    #[serde(default)]
    pub include_deleted: bool,
}

/// Maximum length allowed for a `:serverName` path parameter (bytes).
const SERVER_NAME_MAX_LEN: usize = 512;

/// `GET /v0.1/servers` — list servers with optional search/cursor/limit.
async fn list_servers(
    State(state): State<AppState>,
    Query(query): Query<ListServersQuery>,
) -> Result<impl IntoResponse, ToolError> {
    let Some(store) = state.registry_store.as_ref() else {
        return Ok((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "kind": "sync_in_progress",
                "message": "registry store initializing — try again in a few seconds"
            })),
        )
            .into_response());
    };

    let mut params = StoreListParams {
        cursor: query.cursor,
        limit: query.limit,
        include_deleted: query.include_deleted,
        sort: None,
        search: None,
    };
    if let Some(search) = query.search {
        params = params.with_search(search);
    }

    let paged = store.list_servers(params).await.map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: format!("registry store list_servers: {e}"),
    })?;

    let body = json!({
        "servers": paged.servers,
        "next_cursor": paged.next_cursor,
    });
    Ok((StatusCode::OK, Json(body)).into_response())
}

/// `GET /v0.1/servers/:serverName/versions` — list all versions for a server.
async fn list_versions(
    State(state): State<AppState>,
    Path(server_name): Path<String>,
) -> Result<impl IntoResponse, ToolError> {
    if server_name.len() > SERVER_NAME_MAX_LEN {
        return Err(ToolError::Sdk {
            sdk_kind: "validation_failed".into(),
            message: format!(
                "serverName must be at most {SERVER_NAME_MAX_LEN} characters"
            ),
        });
    }

    let Some(store) = state.registry_store.as_ref() else {
        return Ok((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "kind": "sync_in_progress",
                "message": "registry store initializing — try again in a few seconds"
            })),
        )
            .into_response());
    };

    let versions = store
        .list_versions(&server_name)
        .await
        .map_err(|e| ToolError::Sdk {
            sdk_kind: "internal_error".into(),
            message: format!("registry store list_versions: {e}"),
        })?;

    Ok((StatusCode::OK, Json(json!({ "versions": versions }))).into_response())
}

/// `GET /v0.1/servers/:serverName/versions/:version` — get a single server version.
async fn get_server(
    State(state): State<AppState>,
    Path((server_name, version)): Path<(String, String)>,
) -> Result<impl IntoResponse, ToolError> {
    if server_name.len() > SERVER_NAME_MAX_LEN {
        return Err(ToolError::Sdk {
            sdk_kind: "validation_failed".into(),
            message: format!(
                "serverName must be at most {SERVER_NAME_MAX_LEN} characters"
            ),
        });
    }

    let Some(store) = state.registry_store.as_ref() else {
        return Ok((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "kind": "sync_in_progress",
                "message": "registry store initializing — try again in a few seconds"
            })),
        )
            .into_response());
    };

    match store.get_server(&server_name, &version).await {
        Ok(Some(server)) => Ok((StatusCode::OK, Json(json!(server))).into_response()),
        Ok(None) => Err(ToolError::Sdk {
            sdk_kind: "not_found".into(),
            message: format!("server '{server_name}' version '{version}' not found"),
        }),
        Err(e) => Err(ToolError::Sdk {
            sdk_kind: "internal_error".into(),
            message: format!("registry store get_server: {e}"),
        }),
    }
}

/// Build the `/v0.1` sub-router (routes only — auth middleware applied in `router.rs`).
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/servers", get(list_servers))
        .route("/servers/{serverName}/versions", get(list_versions))
        .route("/servers/{serverName}/versions/{version}", get(get_server))
}
