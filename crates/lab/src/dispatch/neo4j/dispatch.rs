use serde_json::Value;

use lab_apis::neo4j::Neo4jClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, optional_str, require_str, to_json};

use super::catalog::ACTIONS;
use super::client::require_client;
use super::params::{optional_object, statements, tx_mode};

/// Dispatch an action using the service client from env.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("neo4j", ACTIONS)),
        "schema" => {
            let action = require_str(&params, "action")?;
            action_schema(ACTIONS, action)
        }
        _ if !ACTIONS.iter().any(|a| a.name == action) => Err(unknown_action(action)),
        _ => dispatch_with_client(&require_client()?, action, params).await,
    }
}

/// Dispatch an action using a prebuilt client.
pub async fn dispatch_with_client(
    client: &Neo4jClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("neo4j", ACTIONS)),
        "schema" => {
            let action = require_str(&params, "action")?;
            action_schema(ACTIONS, action)
        }
        "cypher.read" => to_json(
            client
                .cypher_read(
                    require_str(&params, "statement")?,
                    optional_object(&params, "parameters")?,
                )
                .await?,
        ),
        "cypher.write" => to_json(
            client
                .cypher_write(
                    require_str(&params, "statement")?,
                    optional_object(&params, "parameters")?,
                )
                .await?,
        ),
        "schema.labels" => to_json(client.labels().await?),
        "schema.relationships" => to_json(client.relationships().await?),
        "schema.constraints" => to_json(client.constraints().await?),
        "schema.indexes" => to_json(client.indexes().await?),
        "db.list" => to_json(client.db_list().await?),
        "db.info" => to_json(client.db_info(optional_str(&params, "database")?).await?),
        "server.status" => to_json(client.server_status().await?),
        "txn.run" => client
            .txn_run(statements(&params)?, tx_mode(&params)?)
            .await
            .map_err(Into::into),
        unknown => Err(unknown_action(unknown)),
    }
}

fn unknown_action(action: &str) -> ToolError {
    ToolError::UnknownAction {
        message: format!("unknown action `{action}` for service `neo4j`"),
        valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
        hint: None,
    }
}
