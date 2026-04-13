use lab_apis::openai::OpenAiClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};

use super::catalog::ACTIONS;
use super::{client, params};

/// Dispatch an action using a pre-built client (called by the API handler).
pub async fn dispatch_with_client(
    client: &OpenAiClient,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("openai", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params_value, "action")?;
            action_schema(ACTIONS, action_name)
        }
        "model.list" => {
            let resp = client.list_models().await?;
            to_json(resp)
        }
        "chat.complete" => {
            let req = params::chat_request_from_params(&params_value)?;
            let resp = client.chat_completion(&req).await?;
            to_json(resp)
        }
        "embed.create" => {
            let req = params::embeddings_request_from_params(&params_value)?;
            let resp = client.create_embeddings(&req).await?;
            to_json(resp)
        }
        "server.health" => {
            client.health().await?;
            Ok(serde_json::json!({ "ok": true }))
        }
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action '{unknown}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

/// Dispatch an action, resolving the client from environment variables.
///
/// Called by MCP and CLI surfaces.
pub async fn dispatch(action: &str, params_value: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("openai", ACTIONS)),
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
