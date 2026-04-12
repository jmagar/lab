use lab_apis::linkding::LinkdingClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};
use crate::dispatch::linkding::{catalog::ACTIONS, client, params};
use crate::dispatch::linkding::params::require_id_u64;

/// Dispatch using a pre-built client (avoids per-request env reads and client construction).
pub async fn dispatch_with_client(
    client: &LinkdingClient,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("linkding", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params_value, "action")?;
            action_schema(ACTIONS, action_name)
        }
        // ── Bookmarks ──────────────────────────────────────────────────────
        "bookmarks.list" => {
            let p = params::bookmark_list_params_from(&params_value)?;
            to_json(client.bookmarks_list(&p).await?)
        }
        "bookmarks.archived.list" => {
            let p = params::bookmark_list_params_from(&params_value)?;
            to_json(client.bookmarks_archived_list(&p).await?)
        }
        "bookmarks.get" => {
            let id = require_id_u64(&params_value)?;
            to_json(client.bookmark_get(id).await?)
        }
        "bookmarks.check" => {
            let url = require_str(&params_value, "url")?;
            to_json(client.bookmark_check(url).await?)
        }
        "bookmarks.create" => {
            let body = params::bookmark_write_from_params(&params_value)?;
            to_json(client.bookmark_create(&body).await?)
        }
        "bookmarks.update" => {
            let id = require_id_u64(&params_value)?;
            let body = params::bookmark_update_from_params(&params_value)?;
            to_json(client.bookmark_update(id, &body).await?)
        }
        "bookmarks.archive" => {
            let id = require_id_u64(&params_value)?;
            to_json(client.bookmark_archive(id).await?)
        }
        "bookmarks.unarchive" => {
            let id = require_id_u64(&params_value)?;
            to_json(client.bookmark_unarchive(id).await?)
        }
        "bookmarks.delete" => {
            let id = require_id_u64(&params_value)?;
            client.bookmark_delete(id).await?;
            Ok(Value::Null)
        }
        // ── Tags ──────────────────────────────────────────────────────────
        "tags.list" => to_json(client.tags_list().await?),
        "tags.get" => {
            let id = require_id_u64(&params_value)?;
            to_json(client.tag_get(id).await?)
        }
        "tags.create" => {
            let body = params::tag_create_from_params(&params_value)?;
            to_json(client.tag_create(&body).await?)
        }
        // ── User ──────────────────────────────────────────────────────────
        "user.profile" => to_json(client.user_profile().await?),
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action '{unknown}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

/// Dispatch one call against the `Linkding` service.
///
/// Returns a surface-neutral `Result<Value, ToolError>` consumed by MCP, CLI,
/// and API.
pub async fn dispatch(action: &str, params_value: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("linkding", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params_value, "action")?;
            return action_schema(ACTIONS, action_name);
        }
        _ if !ACTIONS.iter().any(|a| a.name == action) => {
            return Err(ToolError::UnknownAction {
                message: format!("unknown action '{action}'"),
                valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
                hint: None,
            });
        }
        _ => {}
    }
    dispatch_with_client(&client::require_client()?, action, params_value).await
}
