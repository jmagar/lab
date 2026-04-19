use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::config::UpstreamConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayNameParams {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualServerNameParams {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfigGetParams {
    pub service: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfigSetParams {
    pub service: String,
    pub values: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualServerSurfaceParams {
    pub id: String,
    pub surface: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualServerMcpPolicyParams {
    pub id: String,
    pub allowed_actions: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GatewayTestParams {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub spec: Option<UpstreamConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayAddParams {
    pub spec: UpstreamConfig,
    #[serde(default)]
    pub bearer_token_value: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GatewayUpdatePatch {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub url: Option<Option<String>>,
    #[serde(default)]
    pub command: Option<Option<String>>,
    #[serde(default)]
    pub args: Option<Vec<String>>,
    #[serde(default)]
    pub bearer_token_env: Option<Option<String>>,
    #[serde(default)]
    pub proxy_resources: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_nullable")]
    pub expose_tools: Option<Option<Vec<String>>>,
}

/// Distinguish absent from null for `Option<Option<T>>` patch fields.
///
/// With plain `#[serde(default)]`, serde_json treats both absent fields and
/// explicit `null` as `None`, making it impossible to clear a field via patch.
/// This deserializer wraps the result in `Some(...)` so:
///
/// - absent → `None` (from `#[serde(default)]`)
/// - `null` → `Some(None)` (clear the field)
/// - `["a"]` → `Some(Some(["a"]))` (set the field)
fn deserialize_nullable<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayUpdateParams {
    pub name: String,
    pub patch: GatewayUpdatePatch,
    #[serde(default)]
    pub bearer_token_value: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GatewayStatusParams {
    #[serde(default)]
    pub name: Option<String>,
}
