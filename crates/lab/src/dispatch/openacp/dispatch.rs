use serde_json::Value;

use lab_apis::openacp::OpenAcpClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, optional_str, require_str, to_json};

use super::catalog::ACTIONS;
use super::client::{client_from_instance, require_client};
use super::params;

/// Dispatch an action using a client resolved from env.
pub async fn dispatch(action: &str, params_value: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("openacp", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params_value, "action")?;
            action_schema(ACTIONS, action_name)
        }
        _ if !ACTIONS.iter().any(|a| a.name == action) => Err(unknown_action(action)),
        _ => {
            if params_value.get("instance").is_some() {
                let client = client_from_instance(params::optional_instance(&params_value)?)?;
                dispatch_with_client(&client, action, params_value).await
            } else {
                dispatch_with_client(&require_client()?, action, params_value).await
            }
        }
    }
}

/// Dispatch an action with a prebuilt OpenACP client.
pub async fn dispatch_with_client(
    client: &OpenAcpClient,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("openacp", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params_value, "action")?;
            action_schema(ACTIONS, action_name)
        }
        "system.health" => to_json(client.health().await?),
        "system.version" => to_json(client.version().await?),
        "system.restart" => to_json(client.restart().await?),
        "adapters.list" => to_json(client.adapters_list().await?),
        "sessions.list" => to_json(client.sessions_list().await?),
        "sessions.get" => {
            let id = params::session_id(&params_value)?;
            to_json(client.session_get(id).await?)
        }
        "sessions.create" => {
            let req = params::create_session(&params_value)?;
            to_json(client.session_create(&req).await?)
        }
        "sessions.prompt" => {
            let id = params::session_id(&params_value)?;
            let req = params::prompt(&params_value)?;
            to_json(client.session_prompt(id, &req).await?)
        }
        "sessions.cancel" => {
            let id = params::session_id(&params_value)?;
            client.session_cancel(id).await?;
            Ok(Value::Null)
        }
        "sessions.bypass.set" => {
            let id = params::session_id(&params_value)?;
            let req = params::bypass(&params_value)?;
            to_json(client.session_bypass_set(id, &req).await?)
        }
        "sessions.permission.resolve" => {
            let id = params::session_id(&params_value)?;
            let req = params::permission(&params_value)?;
            to_json(client.session_permission_resolve(id, &req).await?)
        }
        "sessions.archive" => {
            let id = params::session_id(&params_value)?;
            to_json(client.session_archive(id).await?)
        }
        "sessions.adopt" => {
            let req = params::adopt(&params_value)?;
            to_json(client.session_adopt(&req).await?)
        }
        "agents.list" => to_json(client.agents_list().await?),
        "config.get" => to_json(client.config_get().await?),
        "config.editable" => to_json(client.config_editable().await?),
        "config.patch" => {
            let req = params::config_patch(&params_value)?;
            to_json(client.config_patch(&req).await?)
        }
        "topics.list" => {
            let status = optional_str(&params_value, "status")?;
            to_json(client.topics_list(status).await?)
        }
        "topics.delete" => {
            let session_id = params::session_id(&params_value)?;
            let force = params::optional_force(&params_value)?;
            client.topic_delete(session_id, force).await?;
            Ok(Value::Null)
        }
        "topics.cleanup" => {
            let req = params::topics_cleanup(&params_value)?;
            to_json(client.topics_cleanup(&req).await?)
        }
        "tunnel.status" => to_json(client.tunnel_status().await?),
        "tunnel.list" => to_json(client.tunnel_list().await?),
        "tunnel.create" => {
            let req = params::tunnel_create(&params_value)?;
            to_json(client.tunnel_create(&req).await?)
        }
        "tunnel.delete" => {
            let port = params::port(&params_value)?;
            client.tunnel_delete(port).await?;
            Ok(Value::Null)
        }
        "tunnel.delete_all" => to_json(client.tunnel_delete_all().await?),
        "notify.send" => {
            let req = params::notify(&params_value)?;
            to_json(client.notify_send(&req).await?)
        }
        unknown => Err(unknown_action(unknown)),
    }
}

fn unknown_action(action: &str) -> ToolError {
    ToolError::UnknownAction {
        message: format!("unknown action `{action}` for service `openacp`"),
        valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
        hint: None,
    }
}
