use serde::{Deserialize, Serialize};

use crate::config::UpstreamConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayNameParams {
    pub name: String,
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
    #[serde(default)]
    pub expose_tools: Option<Option<Vec<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayUpdateParams {
    pub name: String,
    pub patch: GatewayUpdatePatch,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GatewayStatusParams {
    #[serde(default)]
    pub name: Option<String>,
}
