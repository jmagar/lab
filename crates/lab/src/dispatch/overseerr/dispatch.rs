//! Dispatch routing for the `Overseerr` service.

use lab_apis::overseerr::OverseerrClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};

use super::catalog::ACTIONS;
use super::client::require_client;
use super::params::{create_issue_body, create_request_body, optional_u64, require_u64};

/// Dispatch using a pre-built client (avoids per-request env reads and client construction).
///
/// Called by API handlers with the `AppState`-held client.
pub async fn dispatch_with_client(
    client: &OverseerrClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        // ── Status / Health ───────────────────────────────────────────────
        "health" => {
            client.probe().await?;
            Ok(serde_json::json!({ "ok": true }))
        }
        "status" => to_json(client.status().await?),

        // ── Requests ──────────────────────────────────────────────────────
        "request.list" => {
            let take = optional_u64(&params, "take")?
                .map(|n| u32::try_from(n).unwrap_or(u32::MAX));
            let skip = optional_u64(&params, "skip")?
                .map(|n| u32::try_from(n).unwrap_or(u32::MAX));
            let filter = params
                .get("filter")
                .and_then(Value::as_str);
            let sort = params.get("sort").and_then(Value::as_str);
            let requested_by = optional_u64(&params, "requested_by")?;
            to_json(client.request_list(take, skip, filter, sort, requested_by).await?)
        }
        "request.get" => {
            let id = require_u64(&params, "id")?;
            to_json(client.request_get(id).await?)
        }
        "request.create" => {
            let body = create_request_body(&params)?;
            to_json(client.request_create(&body).await?)
        }
        "request.approve" => {
            let id = require_u64(&params, "id")?;
            to_json(client.request_approve(id).await?)
        }
        "request.decline" => {
            let id = require_u64(&params, "id")?;
            to_json(client.request_decline(id).await?)
        }
        "request.delete" => {
            let id = require_u64(&params, "id")?;
            client.request_delete(id).await?;
            Ok(serde_json::json!({ "deleted": true }))
        }

        // ── Search ────────────────────────────────────────────────────────
        "movie.search" | "tv.search" => {
            let query = require_str(&params, "query")?;
            let page = optional_u64(&params, "page")?
                .map(|n| u32::try_from(n).unwrap_or(u32::MAX));
            to_json(client.search(query, page).await?)
        }

        // ── Movie details ─────────────────────────────────────────────────
        "movie.get" => {
            let tmdb_id = require_u64(&params, "tmdb_id")?;
            to_json(client.movie_get(tmdb_id).await?)
        }

        // ── TV details ────────────────────────────────────────────────────
        "tv.get" => {
            let tmdb_id = require_u64(&params, "tmdb_id")?;
            to_json(client.tv_get(tmdb_id).await?)
        }

        // ── Users ─────────────────────────────────────────────────────────
        "user.list" => {
            let take = optional_u64(&params, "take")?
                .map(|n| u32::try_from(n).unwrap_or(u32::MAX));
            let skip = optional_u64(&params, "skip")?
                .map(|n| u32::try_from(n).unwrap_or(u32::MAX));
            to_json(client.user_list(take, skip).await?)
        }
        "user.get" => {
            let id = require_u64(&params, "id")?;
            to_json(client.user_get(id).await?)
        }

        // ── Issues ────────────────────────────────────────────────────────
        "issue.list" => {
            let take = optional_u64(&params, "take")?
                .map(|n| u32::try_from(n).unwrap_or(u32::MAX));
            let skip = optional_u64(&params, "skip")?
                .map(|n| u32::try_from(n).unwrap_or(u32::MAX));
            let filter = params.get("filter").and_then(Value::as_str);
            let sort = params.get("sort").and_then(Value::as_str);
            to_json(client.issue_list(take, skip, filter, sort).await?)
        }
        "issue.get" => {
            let id = require_u64(&params, "id")?;
            to_json(client.issue_get(id).await?)
        }
        "issue.create" => {
            let body = create_issue_body(&params)?;
            to_json(client.issue_create(&body).await?)
        }
        "issue.comment" => {
            let id = require_u64(&params, "id")?;
            let message = require_str(&params, "message")?;
            to_json(client.issue_comment(id, message).await?)
        }

        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action `{unknown}` for service `overseerr`"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

/// Top-level dispatch entry: handle built-ins then route to service logic.
///
/// Called by MCP and CLI. Uses `require_client()` to resolve credentials.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("overseerr", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params, "action")?;
            action_schema(ACTIONS, action_name)
        }
        _ if !ACTIONS.iter().any(|a| a.name == action) => Err(ToolError::UnknownAction {
            message: format!("unknown action `{action}` for service `overseerr`"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
        _ => dispatch_with_client(&require_client()?, action, params).await,
    }
}
