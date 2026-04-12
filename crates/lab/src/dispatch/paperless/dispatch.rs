use lab_apis::paperless::PaperlessClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};
use crate::dispatch::paperless::{catalog::ACTIONS, client, params};
use crate::dispatch::paperless::params::require_id_u64;

/// Dispatch using a pre-built client (avoids per-request env reads and client construction).
///
/// Called by the HTTP API handler which holds a pre-warmed client in `AppState`.
pub async fn dispatch_with_client(
    client: &PaperlessClient,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        // ── Documents ──────────────────────────────────────────────────────
        "documents.list" => to_json(client.documents_list(&params_value).await?),
        "documents.get" => {
            let id = require_id_u64(&params_value)?;
            to_json(client.document_get(id).await?)
        }
        "documents.metadata" => {
            let id = require_id_u64(&params_value)?;
            to_json(client.document_metadata(id).await?)
        }
        "documents.update" => {
            let id = require_id_u64(&params_value)?;
            let body = params::document_update_from_params(&params_value)?;
            to_json(client.document_update(id, &body).await?)
        }
        "documents.delete" => {
            let id = require_id_u64(&params_value)?;
            client.document_delete(id).await?;
            Ok(Value::Null)
        }
        // ── Tags ───────────────────────────────────────────────────────────
        "tags.list" => to_json(client.tags_list().await?),
        "tags.get" => {
            let id = require_id_u64(&params_value)?;
            to_json(client.tag_get(id).await?)
        }
        "tags.create" => {
            let body = params::tag_create_from_params(&params_value)?;
            to_json(client.tag_create(&body).await?)
        }
        "tags.delete" => {
            let id = require_id_u64(&params_value)?;
            client.tag_delete(id).await?;
            Ok(Value::Null)
        }
        // ── Correspondents ─────────────────────────────────────────────────
        "correspondents.list" => to_json(client.correspondents_list().await?),
        "correspondents.get" => {
            let id = require_id_u64(&params_value)?;
            to_json(client.correspondent_get(id).await?)
        }
        "correspondents.create" => {
            let body = params::correspondent_create_from_params(&params_value)?;
            to_json(client.correspondent_create(&body).await?)
        }
        "correspondents.delete" => {
            let id = require_id_u64(&params_value)?;
            client.correspondent_delete(id).await?;
            Ok(Value::Null)
        }
        // ── Document Types ──────────────────────────────────────────────────
        "document_types.list" => to_json(client.document_types_list().await?),
        "document_types.get" => {
            let id = require_id_u64(&params_value)?;
            to_json(client.document_type_get(id).await?)
        }
        "document_types.create" => {
            let body = params::document_type_create_from_params(&params_value)?;
            to_json(client.document_type_create(&body).await?)
        }
        "document_types.delete" => {
            let id = require_id_u64(&params_value)?;
            client.document_type_delete(id).await?;
            Ok(Value::Null)
        }
        // ── Statistics & Tasks ──────────────────────────────────────────────
        "statistics" => to_json(client.statistics().await?),
        "tasks.list" => to_json(client.tasks_list().await?),
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action '{unknown}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

/// Dispatch one call against the `paperless` service.
///
/// Returns a surface-neutral `Result<Value, ToolError>` consumed by MCP, CLI,
/// and API.
pub async fn dispatch(action: &str, params_value: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("paperless", ACTIONS)),
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

