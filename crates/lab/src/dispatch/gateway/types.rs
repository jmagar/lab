use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GatewayConfigView {
    pub name: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub bearer_token_env: Option<String>,
    #[serde(default)]
    pub proxy_resources: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GatewayRuntimeView {
    pub name: String,
    #[serde(default)]
    pub tool_count: usize,
    #[serde(default)]
    pub resource_count: usize,
    #[serde(default)]
    pub prompt_count: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GatewayView {
    pub config: GatewayConfigView,
    pub runtime: GatewayRuntimeView,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GatewayCatalogDiff {
    #[serde(default)]
    pub tools_changed: bool,
    #[serde(default)]
    pub resources_changed: bool,
    #[serde(default)]
    pub prompts_changed: bool,
}
