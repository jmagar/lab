use lab_apis::notebooklm::NotebookLmClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};

use super::catalog::ACTIONS;
use super::{client, params};

pub async fn dispatch_with_client(
    client: &NotebookLmClient,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("notebooklm", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params_value, "action")?;
            action_schema(ACTIONS, action_name)
        }
        "notebook.list" => to_json(client.list_notebooks().await?),
        "notebook.create" => {
            let title = params::title(&params_value)?;
            to_json(client.create_notebook(title).await?)
        }
        "notebook.get" => {
            let notebook_id = params::notebook_id(&params_value)?;
            to_json(client.get_notebook(notebook_id).await?)
        }
        "notebook.delete" => {
            let notebook_id = params::notebook_id(&params_value)?;
            let deleted = client.delete_notebook(notebook_id).await?;
            Ok(serde_json::json!({ "deleted": deleted }))
        }
        "source.list" => {
            let notebook_id = params::notebook_id(&params_value)?;
            to_json(client.list_sources(notebook_id).await?)
        }
        "source.add_url" => {
            let notebook_id = params::notebook_id(&params_value)?;
            let url = params::url(&params_value)?;
            to_json(client.add_url_source(notebook_id, url).await?)
        }
        "server.health" => {
            client.list_notebooks().await?;
            Ok(serde_json::json!({ "ok": true }))
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
        "help" => return Ok(help_payload("notebooklm", ACTIONS)),
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
