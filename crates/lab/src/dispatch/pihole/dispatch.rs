use serde_json::Value;

use lab_apis::pihole::PiholeClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{
    action_schema, help_payload, optional_str, optional_u32, require_str, to_json,
};

use super::catalog::ACTIONS;
use super::client::require_client;

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("pihole", ACTIONS)),
        "schema" => {
            let action = require_str(&params, "action")?;
            action_schema(ACTIONS, action)
        }
        _ if !ACTIONS.iter().any(|a| a.name == action) => Err(unknown_action(action)),
        _ => dispatch_with_client(&require_client()?, action, params).await,
    }
}

pub async fn dispatch_with_client(
    client: &PiholeClient,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("pihole", ACTIONS)),
        "schema" => {
            let action = require_str(&params, "action")?;
            action_schema(ACTIONS, action)
        }
        "server.summary" => to_json(client.summary().await?),
        "server.settings" => to_json(client.settings().await?),
        "blocking.status" => to_json(client.blocking_status().await?),
        "blocking.set" => {
            let blocking = params
                .get("blocking")
                .and_then(Value::as_bool)
                .ok_or_else(|| ToolError::MissingParam {
                    message: "missing required parameter `blocking`".into(),
                    param: "blocking".into(),
                })?;
            to_json(
                client
                    .blocking_set(blocking, optional_u32(&params, "timer_seconds")?)
                    .await?,
            )
        }
        "querylog.search" => {
            let limit = optional_u32(&params, "limit")?.ok_or_else(|| ToolError::MissingParam {
                message: "missing required parameter `limit`".into(),
                param: "limit".into(),
            })?;
            let offset = optional_u32(&params, "offset")?.unwrap_or(0);
            to_json(client.querylog_search(offset, limit).await?)
        }
        "adlist.list" => to_json(client.adlist_list().await?),
        "adlist.add" => to_json(client.adlist_add(require_str(&params, "address")?).await?),
        "adlist.remove" => to_json(client.adlist_remove(require_str(&params, "id")?).await?),
        "domain.list" => to_json(client.domain_list().await?),
        "domain.add" => {
            let domain = require_str(&params, "domain")?;
            let domain_type =
                optional_u32(&params, "domain_type")?.ok_or_else(|| ToolError::MissingParam {
                    message: "missing required parameter `domain_type`".into(),
                    param: "domain_type".into(),
                })?;
            let domain_type = u8::try_from(domain_type).map_err(|_| ToolError::InvalidParam {
                message: "parameter `domain_type` must fit in u8".into(),
                param: "domain_type".into(),
            })?;
            to_json(
                client
                    .domain_add(domain, domain_type, optional_str(&params, "comment")?)
                    .await?,
            )
        }
        unknown => Err(unknown_action(unknown)),
    }
}

fn unknown_action(action: &str) -> ToolError {
    ToolError::UnknownAction {
        message: format!("unknown action `{action}` for service `pihole`"),
        valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
        hint: None,
    }
}
