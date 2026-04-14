use lab_apis::qdrant::QdrantClient;
use lab_apis::qdrant::types::{
    CreateCollection, CreateIndex, Distance, SearchRequest, UpsertPoint, VectorParams,
};
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};

use super::{catalog::ACTIONS, client};

#[allow(clippy::too_many_lines)]
pub async fn dispatch_with_client(
    client: &QdrantClient,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("qdrant", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params_value, "action")?;
            action_schema(ACTIONS, action_name)
        }
        "server.health" => {
            client.health().await?;
            Ok(serde_json::json!({ "ok": true }))
        }
        "collections.list" => to_json(client.collections_list().await?),
        "collections.get" => {
            let name = super::params::collection_name_from_params(&params_value)?;
            to_json(client.collection_get(name).await?)
        }

        "collection.create" => {
            let name = super::params::collection_name_from_params(&params_value)?;
            let size = crate::dispatch::helpers::require_i64(&params_value, "size")?;
            if size <= 0 {
                return Err(ToolError::InvalidParam {
                    message: "parameter `size` must be a positive integer".into(),
                    param: "size".into(),
                });
            }
            let distance_str = require_str(&params_value, "distance")?;
            let distance = match distance_str {
                "Cosine" => Distance::Cosine,
                "Euclid" => Distance::Euclid,
                "Dot" => Distance::Dot,
                "Manhattan" => Distance::Manhattan,
                other => {
                    return Err(ToolError::InvalidParam {
                        message: format!(
                            "parameter `distance` must be one of Cosine, Euclid, Dot, Manhattan; got `{other}`"
                        ),
                        param: "distance".into(),
                    });
                }
            };
            let body = CreateCollection {
                vectors: VectorParams {
                    #[allow(clippy::cast_sign_loss)]
                    size: size as u64,
                    distance,
                },
            };
            to_json(client.collection_create(name, &body).await?)
        }

        "collection.delete" => {
            let name = super::params::collection_name_from_params(&params_value)?;
            client.collection_delete(name).await?;
            Ok(serde_json::json!({ "ok": true }))
        }

        "point.upsert" => {
            let collection = require_str(&params_value, "collection")?;
            let raw_points = params_value
                .get("points")
                .ok_or_else(|| ToolError::MissingParam {
                    message: "missing required parameter `points`".into(),
                    param: "points".into(),
                })?;
            let points: Vec<UpsertPoint> =
                serde_json::from_value(raw_points.clone()).map_err(|e| {
                    ToolError::InvalidParam {
                        message: format!("parameter `points` is not valid: {e}"),
                        param: "points".into(),
                    }
                })?;
            to_json(client.point_upsert_batched(collection, points).await?)
        }

        "point.search" => {
            let collection = require_str(&params_value, "collection")?;
            let raw_vector = params_value
                .get("vector")
                .ok_or_else(|| ToolError::MissingParam {
                    message: "missing required parameter `vector`".into(),
                    param: "vector".into(),
                })?;
            let vector: Vec<f32> = serde_json::from_value(raw_vector.clone()).map_err(|e| {
                ToolError::InvalidParam {
                    message: format!("parameter `vector` is not valid: {e}"),
                    param: "vector".into(),
                }
            })?;
            let limit = crate::dispatch::helpers::require_i64(&params_value, "limit")?;
            if limit <= 0 {
                return Err(ToolError::InvalidParam {
                    message: "parameter `limit` must be a positive integer".into(),
                    param: "limit".into(),
                });
            }
            let filter = params_value.get("filter").cloned();
            let with_payload = params_value.get("with_payload").and_then(Value::as_bool);
            let score_threshold = params_value.get("score_threshold").and_then(|v| {
                v.as_f64().map(|f| {
                    #[allow(clippy::cast_possible_truncation)]
                    {
                        f as f32
                    }
                })
            });
            let req = SearchRequest {
                vector,
                #[allow(clippy::cast_sign_loss)]
                limit: limit as u64,
                filter,
                with_payload,
                score_threshold,
            };
            to_json(client.point_search(collection, &req).await?)
        }

        "point.query" => {
            let collection = require_str(&params_value, "collection")?;
            let query = params_value
                .get("query")
                .cloned()
                .unwrap_or(Value::Object(serde_json::Map::new()));
            to_json(client.point_query(collection, &query).await?)
        }

        "point.scroll" => {
            let collection = require_str(&params_value, "collection")?;
            let body = params_value
                .get("scroll")
                .cloned()
                .unwrap_or(Value::Object(serde_json::Map::new()));
            to_json(client.point_scroll(collection, &body).await?)
        }

        "point.count" => {
            let collection = require_str(&params_value, "collection")?;
            let body = params_value.get("filter").map_or_else(
                || Value::Object(serde_json::Map::new()),
                |filter| serde_json::json!({ "filter": filter }),
            );
            to_json(client.point_count(collection, &body).await?)
        }

        "point.delete" => {
            let collection = require_str(&params_value, "collection")?;
            // Build the delete body from points and/or filter
            let mut body = serde_json::Map::new();
            if let Some(points) = params_value.get("points") {
                body.insert("points".into(), points.clone());
            }
            if let Some(filter) = params_value.get("filter") {
                body.insert("filter".into(), filter.clone());
            }
            if body.is_empty() {
                return Err(ToolError::MissingParam {
                    message: "at least one of `points` or `filter` must be provided".into(),
                    param: "points".into(),
                });
            }
            to_json(
                client
                    .point_delete(collection, &Value::Object(body))
                    .await?,
            )
        }

        "snapshot.create" => {
            let collection = require_str(&params_value, "collection")?;
            to_json(client.snapshot_create(collection).await?)
        }

        "index.create" => {
            let collection = require_str(&params_value, "collection")?;
            let field_name = require_str(&params_value, "field_name")?.to_string();
            let field_schema = params_value.get("field_schema").cloned().ok_or_else(|| {
                ToolError::MissingParam {
                    message: "missing required parameter `field_schema`".into(),
                    param: "field_schema".into(),
                }
            })?;
            let req = CreateIndex {
                field_name,
                field_schema,
            };
            to_json(client.index_create(collection, &req).await?)
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
        "help" => return Ok(help_payload("qdrant", ACTIONS)),
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
