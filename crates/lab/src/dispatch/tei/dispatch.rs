use lab_apis::tei::types::{RerankRequest, SimilarityRequest, SparseEmbedRequest, TokenizeRequest};
use lab_apis::tei::TeiClient;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};

use super::{catalog::ACTIONS, client, params};

#[allow(clippy::too_many_lines)]
pub async fn dispatch_with_client(
    client: &TeiClient,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("tei", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params_value, "action")?;
            action_schema(ACTIONS, action_name)
        }
        "server.health" => {
            client.health().await?;
            Ok(serde_json::json!({ "ok": true }))
        }
        "server.info" => to_json(client.model_info().await?),
        "embed.create" => {
            let request = params::embed_request_from_params(&params_value)?;
            to_json(client.embed(&request).await?)
        }
        "embed.rerank" => {
            let query = require_str(&params_value, "query")?.to_string();
            let texts_raw = params_value["texts"]
                .as_array()
                .ok_or_else(|| ToolError::MissingParam {
                    message: "missing required parameter `texts`".into(),
                    param: "texts".into(),
                })?;
            if texts_raw.len() > 100 {
                return Err(ToolError::Sdk {
                    sdk_kind: "validation_error".to_string(),
                    message: format!(
                        "embed.rerank: texts len {} exceeds 100 limit",
                        texts_raw.len()
                    ),
                });
            }
            let texts = texts_raw
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(ToOwned::to_owned)
                        .ok_or_else(|| ToolError::InvalidParam {
                            message: "parameter `texts` must be an array of strings".into(),
                            param: "texts".into(),
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let truncate = params_value.get("truncate").and_then(Value::as_bool);
            let raw_scores = params_value.get("raw_scores").and_then(Value::as_bool);
            let request = RerankRequest {
                query,
                texts,
                truncate,
                raw_scores,
                return_text: None,
            };
            to_json(client.rerank(&request).await?)
        }
        "embed.tokenize" => {
            let inputs = params_value
                .get("inputs")
                .cloned()
                .ok_or_else(|| ToolError::MissingParam {
                    message: "missing required parameter `inputs`".into(),
                    param: "inputs".into(),
                })?;
            let add_special_tokens = params_value
                .get("add_special_tokens")
                .and_then(Value::as_bool);
            let request = TokenizeRequest {
                inputs,
                add_special_tokens,
            };
            to_json(client.tokenize(&request).await?)
        }
        "embed.similarity" => {
            let inputs_raw = params_value["inputs"]
                .as_array()
                .ok_or_else(|| ToolError::MissingParam {
                    message: "missing required parameter `inputs`".into(),
                    param: "inputs".into(),
                })?;
            let inputs = inputs_raw
                .iter()
                .enumerate()
                .map(|(i, pair)| {
                    let arr = pair.as_array().ok_or_else(|| ToolError::InvalidParam {
                        message: format!("inputs[{i}] must be a [string, string] pair"),
                        param: "inputs".into(),
                    })?;
                    if arr.len() != 2 {
                        return Err(ToolError::InvalidParam {
                            message: format!(
                                "inputs[{i}] must be a 2-element [string, string] pair, got {} elements",
                                arr.len()
                            ),
                            param: "inputs".into(),
                        });
                    }
                    let a = arr[0].as_str().ok_or_else(|| ToolError::InvalidParam {
                        message: format!("inputs[{i}][0] must be a string"),
                        param: "inputs".into(),
                    })?;
                    let b = arr[1].as_str().ok_or_else(|| ToolError::InvalidParam {
                        message: format!("inputs[{i}][1] must be a string"),
                        param: "inputs".into(),
                    })?;
                    Ok([a.to_owned(), b.to_owned()])
                })
                .collect::<Result<Vec<_>, _>>()?;
            let request = SimilarityRequest { inputs };
            to_json(client.similarity(&request).await?)
        }
        "embed.sparse" => {
            let inputs = params_value
                .get("inputs")
                .cloned()
                .ok_or_else(|| ToolError::MissingParam {
                    message: "missing required parameter `inputs`".into(),
                    param: "inputs".into(),
                })?;
            let truncate = params_value.get("truncate").and_then(Value::as_bool);
            let request = SparseEmbedRequest { inputs, truncate };
            to_json(client.embed_sparse(&request).await?)
        }
        "embed.openai" => {
            let body = params_value
                .get("body")
                .cloned()
                .ok_or_else(|| ToolError::MissingParam {
                    message: "missing required parameter `body`".into(),
                    param: "body".into(),
                })?;
            to_json(client.openai_embed(&body).await?)
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
        "help" => return Ok(help_payload("tei", ACTIONS)),
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
