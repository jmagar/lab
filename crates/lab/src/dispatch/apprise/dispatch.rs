use lab_apis::apprise::AppriseClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str};

use super::{catalog::ACTIONS, client, params};

pub async fn dispatch_with_client(
    client: &AppriseClient,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("apprise", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params_value, "action")?;
            action_schema(ACTIONS, action_name)
        }
        "server.health" => {
            client.health().await?;
            Ok(serde_json::json!({ "ok": true }))
        }
        "notify.send" => {
            let request = params::notify_request_from_params(&params_value, &[])?;
            client.notify(&request).await?;
            Ok(Value::Null)
        }
        "notify.key.send" => {
            let key = require_str(&params_value, "key")?;
            let request = params::notify_request_from_params(&params_value, &["key"])?;
            client.notify_key(key, &request).await?;
            Ok(Value::Null)
        }
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action '{unknown}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

pub async fn dispatch(action: &str, params_value: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("apprise", ACTIONS)),
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
