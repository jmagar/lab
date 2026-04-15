use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Surface-neutral notification handle for catalog changes.
///
/// The dispatch layer calls this after gateway reload/add/remove to inform
/// connected transports (e.g. MCP peers) that tools, resources, or prompts
/// have changed.  The concrete implementation lives in the MCP surface
/// (`mcp/server.rs`) so that `rmcp::Peer` never leaks into dispatch.
#[derive(Clone, Debug)]
pub struct CatalogChangeNotifier {
    tx: mpsc::UnboundedSender<GatewayCatalogDiff>,
}

impl CatalogChangeNotifier {
    #[must_use]
    pub fn new(tx: mpsc::UnboundedSender<GatewayCatalogDiff>) -> Self {
        Self { tx }
    }

    pub fn notify_catalog_changes(&self, diff: &GatewayCatalogDiff) {
        let _ = self.tx.send(diff.clone());
    }
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
    #[serde(default)]
    pub last_error: Option<String>,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceConfigFieldView {
    pub name: String,
    #[serde(default)]
    pub present: bool,
    #[serde(default)]
    pub secret: bool,
    #[serde(default)]
    pub value_preview: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceConfigView {
    pub service: String,
    #[serde(default)]
    pub configured: bool,
    #[serde(default)]
    pub fields: Vec<ServiceConfigFieldView>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VirtualServerMcpPolicyView {
    #[serde(default)]
    pub allowed_actions: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceActionView {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub destructive: bool,
}
