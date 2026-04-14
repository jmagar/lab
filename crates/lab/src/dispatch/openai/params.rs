use lab_apis::openai::types::{
    ChatCompletionRequest, ChatMessage, ChatRole, EmbeddingsInput, EmbeddingsRequest, ModelId,
};
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::require_str;

/// Build a [`ChatCompletionRequest`] from dispatch params.
///
/// # Errors
/// Returns `MissingParam` for absent required fields, `InvalidParam` for
/// values with the wrong type or shape.
pub fn chat_request_from_params(params: &Value) -> Result<ChatCompletionRequest, ToolError> {
    let model = require_str(params, "model")?;
    let messages_val = params.get("messages").ok_or_else(|| ToolError::MissingParam {
        message: "missing required parameter `messages`".to_string(),
        param: "messages".to_string(),
    })?;

    let messages = parse_messages(messages_val)?;

    let temperature = params
        .get("temperature")
        .map(|v| {
            v.as_f64()
                .map(|f| {
                    #[allow(clippy::cast_possible_truncation)]
                    { f as f32 }
                })
                .ok_or_else(|| ToolError::InvalidParam {
                    message: "parameter `temperature` must be a number".to_string(),
                    param: "temperature".to_string(),
                })
        })
        .transpose()?;

    let max_tokens = params
        .get("max_tokens")
        .map(|v| {
            v.as_u64()
                .and_then(|n| u32::try_from(n).ok())
                .ok_or_else(|| ToolError::InvalidParam {
                    message: "parameter `max_tokens` must be a non-negative integer".to_string(),
                    param: "max_tokens".to_string(),
                })
        })
        .transpose()?;

    Ok(ChatCompletionRequest {
        model: ModelId(model.to_string()),
        messages,
        temperature,
        max_tokens,
        stream: None,
    })
}

/// Parse the `messages` array from a JSON value.
fn parse_messages(val: &Value) -> Result<Vec<ChatMessage>, ToolError> {
    // Accept a JSON string encoding of an array (callers may stringify).
    let arr = match val {
        Value::Array(a) => a.as_slice(),
        Value::String(s) => {
            return serde_json::from_str::<Vec<ChatMessage>>(s).map_err(|e| {
                ToolError::InvalidParam {
                    message: format!("parameter `messages` is not valid JSON: {e}"),
                    param: "messages".to_string(),
                }
            });
        }
        _ => {
            return Err(ToolError::InvalidParam {
                message: "parameter `messages` must be an array".to_string(),
                param: "messages".to_string(),
            });
        }
    };

    arr.iter()
        .enumerate()
        .map(|(i, m)| {
            let role_str = m
                .get("role")
                .and_then(Value::as_str)
                .ok_or_else(|| ToolError::InvalidParam {
                    message: format!("messages[{i}].role must be a string"),
                    param: "messages".to_string(),
                })?;
            let role = match role_str {
                "system" => ChatRole::System,
                "user" => ChatRole::User,
                "assistant" => ChatRole::Assistant,
                "tool" => ChatRole::Tool,
                other => {
                    return Err(ToolError::InvalidParam {
                        message: format!(
                            "messages[{i}].role `{other}` is not valid; \
                             use system, user, assistant, or tool"
                        ),
                        param: "messages".to_string(),
                    });
                }
            };
            let content = m
                .get("content")
                .and_then(Value::as_str)
                .ok_or_else(|| ToolError::InvalidParam {
                    message: format!("messages[{i}].content must be a string"),
                    param: "messages".to_string(),
                })?;
            Ok(ChatMessage {
                role,
                content: content.to_string(),
            })
        })
        .collect()
}

/// Build an [`EmbeddingsRequest`] from dispatch params.
///
/// # Errors
/// Returns `MissingParam` when neither `input` nor `inputs` is supplied.
pub fn embeddings_request_from_params(params: &Value) -> Result<EmbeddingsRequest, ToolError> {
    let model = require_str(params, "model")?;

    // Collect input strings from `inputs` (array) or `input` (scalar).
    let input = match params.get("inputs") {
        Some(Value::Array(arr)) => {
            let strings: Result<Vec<String>, _> = arr
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    v.as_str()
                        .map(ToOwned::to_owned)
                        .ok_or_else(|| ToolError::InvalidParam {
                            message: format!("inputs[{i}] must be a string"),
                            param: "inputs".to_string(),
                        })
                })
                .collect();
            EmbeddingsInput::Batch(strings?)
        }
        Some(Value::String(s)) => EmbeddingsInput::Batch(vec![s.clone()]),
        Some(_) => {
            return Err(ToolError::InvalidParam {
                message: "parameter `inputs` must be a string or array of strings".to_string(),
                param: "inputs".to_string(),
            });
        }
        None => match params.get("input").and_then(Value::as_str) {
            Some(s) => EmbeddingsInput::Single(s.to_string()),
            None => {
                return Err(ToolError::MissingParam {
                    message: "missing required parameter `input` or `inputs`".to_string(),
                    param: "input".to_string(),
                });
            }
        },
    };

    let dimensions = params
        .get("dimensions")
        .map(|v| {
            v.as_u64()
                .and_then(|n| u32::try_from(n).ok())
                .ok_or_else(|| ToolError::InvalidParam {
                    message: "parameter `dimensions` must be a non-negative integer".to_string(),
                    param: "dimensions".to_string(),
                })
        })
        .transpose()?;

    Ok(EmbeddingsRequest {
        model: ModelId(model.to_string()),
        input,
        dimensions,
    })
}
