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
        "indexer.list" => to_json(client.indexers_list().await?),
        "indexer.get" => {
            let id = params::require_id(&params_value)?;
            to_json(client.indexer_get(id).await?)
        }
        "indexer.delete" => {
            let id = params::require_id(&params_value)?;
            client.indexer_delete(id).await?;
            Ok(Value::Null)
        }
        "indexer.test" => {
            let id = params::require_id(&params_value)?;
            to_json(client.indexer_test(id).await?)
        }
        "indexer.testall" => to_json(client.indexers_testall().await?),
        "indexer.categories" => to_json(client.indexer_categories().await?),
        "history.list" => {
            let query = params::history_query_from_params(&params_value)?;
            to_json(client.history_list(&query).await?)
        }
        "application.list" => to_json(client.applications_list().await?),
        "application.get" => {
            let id = params::require_id(&params_value)?;
            to_json(client.application_get(id).await?)
        }
        "application.delete" => {
            let id = params::require_id(&params_value)?;
            client.application_delete(id).await?;
            Ok(Value::Null)
        }
        "indexer.edit" => {
            let id = params::require_id(&params_value)?;
            let body = params::require_body(&params_value)?;
            to_json(client.indexer_edit(id, &body).await?)
        }
        "indexer.add" => {
            let body = params::require_body(&params_value)?;
            to_json(client.indexer_add(&body).await?)
        }
        "indexer.stats" => to_json(client.indexer_stats().await?),
        "indexer.status" => to_json(client.indexer_status().await?),
        "indexer.search" => {
            let query = params::require_query_str(&params_value)?;
            let indexer_ids = params::optional_i64_array(&params_value, "indexer_ids")?;
            let categories = params::optional_i64_array(&params_value, "categories")?;
            let search_type = params::optional_search_type(&params_value)?;
            to_json(
                client
                    .indexer_search(
                        &query,
                        &indexer_ids,
                        &categories,
                        search_type.as_deref(),
                    )
                    .await?,
            )
        }
        "indexer.grab" => {
            let guid = params::require_guid(&params_value)?;
            to_json(client.indexer_grab(&guid).await?)
        }
        "history.indexer" => {
            let id = params::require_id(&params_value)?;
            to_json(client.history_indexer(id).await?)
        }
        "application.add" => {
            let body = params::require_body(&params_value)?;
            to_json(client.application_add(&body).await?)
        }
        "system.restart" => {
            client.system_restart().await?;
            Ok(Value::Null)
        }
        "system.backup" => to_json(client.system_backup().await?),
        "tag.list" => to_json(client.tag_list().await?),
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
