use lab_apis::bytestash::ByteStashClient;
use serde_json::Value;

use crate::dispatch::bytestash::{catalog::ACTIONS, client, params};
use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};

/// Dispatch using a pre-built client (avoids per-request env reads and client construction).
pub async fn dispatch_with_client(
    client: &ByteStashClient,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        "auth.config" => to_json(client.auth_config().await?),
        "auth.register" => {
            let body = params::credentials_from_params(&params_value)?;
            to_json(client.auth_register(&body).await?)
        }
        "auth.login" => {
            let body = params::credentials_from_params(&params_value)?;
            to_json(client.auth_login(&body).await?)
        }
        "snippets.list" => to_json(client.snippets_list().await?),
        "snippets.get" => {
            let id = require_str(&params_value, "id")?;
            to_json(client.snippet_get(id).await?)
        }
        "snippets.create" => {
            let body = params::snippet_write_from_params(&params_value)?;
            to_json(client.snippets_create(&body).await?)
        }
        "snippets.update" => {
            let id = require_str(&params_value, "id")?;
            let body = params::snippet_write_from_params(&params_value)?;
            to_json(client.snippets_update(id, &body).await?)
        }
        "snippets.delete" => {
            let id = require_str(&params_value, "id")?;
            client.snippets_delete(id).await?;
            Ok(Value::Null)
        }
        "snippets.public.list" => to_json(client.snippets_public_list().await?),
        "snippets.public.get" => {
            let id = require_str(&params_value, "id")?;
            to_json(client.snippets_public_get(id).await?)
        }
        "snippets.share.create" => {
            let body = params::share_create_from_params(&params_value)?;
            to_json(client.snippets_share_create(&body).await?)
        }
        "snippets.share.get" => {
            let share_id = require_str(&params_value, "share_id")?;
            to_json(client.snippets_share_get(share_id).await?)
        }
        "categories.list" => to_json(client.categories_list().await?),
        "users.list" => to_json(client.users_list().await?),
        "users.toggle-active" => {
            let id = require_str(&params_value, "id")?;
            to_json(client.users_toggle_active(id).await?)
        }
        "users.delete" => {
            let id = require_str(&params_value, "id")?;
            client.users_delete(id).await?;
            Ok(Value::Null)
        }
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action '{unknown}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

/// Dispatch one call against the `ByteStash` service.
///
/// Returns a surface-neutral `Result<Value, ToolError>` consumed by MCP, CLI,
/// and API.
pub async fn dispatch(action: &str, params_value: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("bytestash", ACTIONS)),
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
