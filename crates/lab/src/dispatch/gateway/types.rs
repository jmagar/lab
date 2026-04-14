use std::future::Future;
use std::pin::Pin;

use serde::{Deserialize, Serialize};

/// Surface-neutral notification trait for catalog changes.
///
/// The dispatch layer calls this after gateway reload/add/remove to inform
/// connected transports (e.g. MCP peers) that tools, resources, or prompts
/// have changed.  The concrete implementation lives in the MCP surface
/// (`mcp/server.rs`) so that `rmcp::Peer` never leaks into dispatch.
pub trait CatalogChangeNotifier: Send + Sync {
    fn notify_catalog_changes<'a>(
        &'a self,
        diff: &'a GatewayCatalogDiff,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
}

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
