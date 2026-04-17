//! Top-level action router for the `logs` dispatch service.

use serde_json::Value;

use super::catalog::ACTIONS;
use super::client;
use super::params::{parse_search_params, parse_tail_params};
use super::types::LogSystem;
use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("logs", ACTIONS)),
        "schema" => {
            let a = require_str(&params, "action")?;
            action_schema(ACTIONS, a)
        }
        other => {
            validate_action(other)?;
            let system = client::require_system()?;
            dispatch_with_system(&system, other, params).await
        }
    }
}

pub async fn dispatch_with_system(
    system: &LogSystem,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("logs", ACTIONS)),
        "schema" => {
            let a = require_str(&params, "action")?;
            action_schema(ACTIONS, a)
        }
        "logs.search" => {
            let q = parse_search_params(params)?;
            let result = system.search(q).await?;
            to_json(result)
        }
        "logs.tail" => {
            let r = parse_tail_params(params)?;
            let result = system.tail(r).await?;
            to_json(result)
        }
        "logs.stats" => {
            let stats = system.stats().await?;
            to_json(stats)
        }
        "logs.stream" => Err(ToolError::Sdk {
            sdk_kind: "not_found".to_string(),
            message:
                "live push is HTTP SSE only; connect to GET /v1/logs/stream to receive events"
                    .to_string(),
        }),
        unknown => Err(unknown_action_error(unknown)),
    }
}

fn validate_action(action: &str) -> Result<(), ToolError> {
    if ACTIONS.iter().any(|a| a.name == action) {
        Ok(())
    } else {
        Err(unknown_action_error(action))
    }
}

fn unknown_action_error(action: &str) -> ToolError {
    let valid: Vec<String> = ACTIONS.iter().map(|a| a.name.to_string()).collect();
    let hint = closest(&valid, action);
    ToolError::UnknownAction {
        message: format!("unknown action `{action}` for service `logs`"),
        valid,
        hint,
    }
}

fn closest(valid: &[String], action: &str) -> Option<String> {
    valid
        .iter()
        .filter(|v| v.len().abs_diff(action.len()) <= 2)
        .min_by_key(|v| levenshtein(v, action))
        .filter(|v| levenshtein(v, action) <= 3)
        .cloned()
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        curr[0] = i;
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn help_returns_catalog_object() {
        let v = dispatch("help", serde_json::json!({})).await.unwrap();
        assert!(v.is_object());
        assert_eq!(v["service"], "logs");
    }

    #[tokio::test]
    async fn schema_returns_action_shape() {
        let v = dispatch("schema", serde_json::json!({"action": "logs.search"}))
            .await
            .unwrap();
        assert!(v.is_object());
    }

    #[tokio::test]
    async fn unknown_action_returns_kind() {
        let e = dispatch("logs.serch", serde_json::json!({}))
            .await
            .unwrap_err();
        assert_eq!(e.kind(), "unknown_action");
    }
}
