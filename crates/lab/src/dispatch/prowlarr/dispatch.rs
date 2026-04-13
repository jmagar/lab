use lab_apis::prowlarr::ProwlarrClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, to_json};
use crate::dispatch::prowlarr::{catalog::ACTIONS, client, params};

/// Dispatch using a pre-built client (avoids per-request env reads and client construction).
pub async fn dispatch_with_client(
    client: &ProwlarrClient,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        "indexers.list" => to_json(client.indexers_list().await?),
        "indexers.get" => {
            let id = params::require_id(&params_value)?;
            to_json(client.indexer_get(id).await?)
        }
        "indexers.delete" => {
            let id = params::require_id(&params_value)?;
            client.indexer_delete(id).await?;
            Ok(Value::Null)
        }
        "indexers.test" => {
            let id = params::require_id(&params_value)?;
            to_json(client.indexer_test(id).await?)
        }
        "indexers.testall" => to_json(client.indexers_testall().await?),
        "indexers.categories" => to_json(client.indexer_categories().await?),
        "history.list" => {
            let query = params::history_query_from_params(&params_value)?;
            to_json(client.history_list(&query).await?)
        }
        "applications.list" => to_json(client.applications_list().await?),
        "applications.get" => {
            let id = params::require_id(&params_value)?;
            to_json(client.application_get(id).await?)
        }
        "applications.delete" => {
            let id = params::require_id(&params_value)?;
            client.application_delete(id).await?;
            Ok(Value::Null)
        }
        "system.status" => to_json(client.system_status().await?),
        "system.health" => to_json(client.system_health().await?),
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action '{unknown}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

/// Dispatch one call against the `Prowlarr` service.
///
/// Returns a surface-neutral `Result<Value, ToolError>` consumed by MCP, CLI,
/// and API.
pub async fn dispatch(action: &str, params_value: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("prowlarr", ACTIONS)),
        "schema" => {
            let action_name = crate::dispatch::helpers::require_str(&params_value, "action")?;
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
