use lab_apis::plex::PlexClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};

use super::catalog::ACTIONS;
use super::client::require_client;
use super::params;

/// Dispatch using a pre-built client (avoids per-request env reads and client construction).
pub async fn dispatch_with_client(
    client: &PlexClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        // ── Server ────────────────────────────────────────────────────────────
        "server.info" => to_json(client.server_info().await?),
        "server.capabilities" => to_json(client.server_capabilities().await?),

        // ── Library ───────────────────────────────────────────────────────────
        "library.list" => to_json(client.library_list().await?),
        "library.get" => {
            let section_id = params::require_key(&params, "section_id")?;
            to_json(client.library_get(section_id).await?)
        }
        "library.scan" => {
            let section_id = params::require_key(&params, "section_id")?;
            to_json(client.library_scan(section_id).await?)
        }
        "library.refresh" => {
            let section_id = params::require_key(&params, "section_id")?;
            to_json(client.library_refresh(section_id).await?)
        }

        // ── Media ─────────────────────────────────────────────────────────────
        "media.search" => {
            let p = params::search_params_from(&params)?;
            to_json(client.media_search(&p).await?)
        }
        "media.get" => {
            let rating_key = params::require_key(&params, "rating_key")?;
            to_json(client.media_get(rating_key).await?)
        }

        // ── Sessions ──────────────────────────────────────────────────────────
        "session.list" => to_json(client.session_list().await?),
        "session.terminate" => {
            let session_id = params::require_key(&params, "session_id")?;
            let reason = params::optional_string(&params, "reason")?;
            to_json(
                client
                    .session_terminate(session_id, reason.as_deref())
                    .await?,
            )
        }

        // ── Playlists ─────────────────────────────────────────────────────────
        "playlist.list" => to_json(client.playlist_list().await?),
        "playlist.get" => {
            let playlist_id = params::require_key(&params, "playlist_id")?;
            to_json(client.playlist_get(playlist_id).await?)
        }
        "playlist.create" => {
            let title = params::require_key(&params, "title")?;
            let playlist_type = params::require_key(&params, "playlist_type")?;
            let uri = params::optional_string(&params, "uri")?;
            to_json(
                client
                    .playlist_create(title, playlist_type, uri.as_deref())
                    .await?,
            )
        }
        "playlist.delete" => {
            let playlist_id = params::require_key(&params, "playlist_id")?;
            client.playlist_delete(playlist_id).await?;
            Ok(Value::Null)
        }

        // ── Health ────────────────────────────────────────────────────────────
        "health" => {
            client.probe().await?;
            Ok(serde_json::json!({ "status": "ok" }))
        }

        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action '{unknown}' for service `plex`"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

/// Dispatch one call against the `plex` service.
///
/// Returns a surface-neutral `Result<Value, ToolError>` consumed by MCP, CLI,
/// and API.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("plex", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params, "action")?;
            return action_schema(ACTIONS, action_name);
        }
        _ if !ACTIONS.iter().any(|a| a.name == action) => {
            return Err(ToolError::UnknownAction {
                message: format!("unknown action '{action}' for service `plex`"),
                valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
                hint: None,
            });
        }
        _ => {}
    }
    dispatch_with_client(&require_client()?, action, params).await
}
