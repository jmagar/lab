//! `UpstreamPool` — manages connections to upstream MCP servers.
//!
//! Connects to configured upstreams via HTTP (`StreamableHttpClientTransport`)
//! or stdio (child process), discovers their tools, and caches schemas.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use futures::stream::FuturesUnordered;
use rmcp::model::{CallToolRequestParams, CallToolResult, GetPromptRequestParams, GetPromptResult};
use rmcp::transport::streamable_http_client::{
    StreamableHttpClientTransportConfig, StreamableHttpClientWorker,
};
use rmcp::{RoleClient, ServiceExt};
use serde_json::Value;
use tokio::sync::RwLock;

use crate::config::UpstreamConfig;

use super::types;
use super::types::{
    ToolExposurePolicy, UpstreamCapability, UpstreamEntry, UpstreamHealth, UpstreamTool,
    UpstreamToolExposureRow,
};

/// Per-upstream timeout for initial discovery (`list_tools`).
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(15);

/// Default maximum response size from upstream servers (10 MB).
const DEFAULT_MAX_RESPONSE_BYTES: usize = 10 * 1024 * 1024;

/// Read the max response size from env or use the default.
fn max_response_bytes() -> usize {
    std::env::var("LAB_UPSTREAM_MAX_RESPONSE_BYTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_MAX_RESPONSE_BYTES)
}

/// Collect upstream peers for a capability in deterministic name order.
async fn routable_upstream_peers(
    pool: &UpstreamPool,
    capability: UpstreamCapability,
) -> Vec<(String, rmcp::service::Peer<RoleClient>)> {
    let mut names: Vec<String> = {
        let catalog = pool.catalog.read().await;
        let mut names = match capability {
            UpstreamCapability::Resources => {
                let resource_names = pool.resource_upstreams.read().await;
                resource_names
                    .iter()
                    .filter(|name| {
                        catalog
                            .get(*name)
                            .is_some_and(|entry| entry.health_for(capability).is_routable())
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            }
            UpstreamCapability::Tools | UpstreamCapability::Prompts => catalog
                .iter()
                .filter(|(_, entry)| entry.health_for(capability).is_routable())
                .map(|(name, _)| name.clone())
                .collect::<Vec<_>>(),
        };
        names.sort_unstable();
        names.dedup();
        names
    };

    let connections = pool.connections.read().await;
    names
        .drain(..)
        .filter_map(|name| connections.get(&name).map(|conn| (name, conn.peer.clone())))
        .collect()
}

/// Merge upstream prompts deterministically and return the winning owner for each prompt.
fn merge_upstream_prompts(
    builtin_names: &[&str],
    mut upstream_prompts: Vec<(String, Vec<rmcp::model::Prompt>)>,
) -> (Vec<rmcp::model::Prompt>, HashMap<String, String>) {
    upstream_prompts.sort_unstable_by(|left, right| left.0.cmp(&right.0));

    let mut prompts = Vec::new();
    let mut owners = HashMap::new();
    let mut seen_names: std::collections::HashSet<String> = builtin_names
        .iter()
        .map(|name| (*name).to_string())
        .collect();

    for (upstream_name, upstream_prompts) in upstream_prompts {
        for prompt in upstream_prompts {
            let prompt_name = prompt.name.to_string();
            if seen_names.insert(prompt_name.clone()) {
                owners.insert(prompt_name, upstream_name.clone());
                prompts.push(prompt);
            } else {
                tracing::warn!(
                    upstream = %upstream_name,
                    prompt = %prompt.name,
                    "duplicate prompt name encountered while merging upstream prompts"
                );
            }
        }
    }

    (prompts, owners)
}

/// Normalize a proxied resource read so its contents use the gateway URI.
fn normalize_resource_result_uri(
    mut result: rmcp::model::ReadResourceResult,
    gateway_uri: &str,
) -> rmcp::model::ReadResourceResult {
    for content in &mut result.contents {
        match content {
            rmcp::model::ResourceContents::TextResourceContents { uri, .. }
            | rmcp::model::ResourceContents::BlobResourceContents { uri, .. } => {
                *uri = gateway_uri.to_string();
            }
        }
    }

    result
}

/// Upstream connection pool — holds live connections and discovered tool catalogs.
#[derive(Clone)]
pub struct UpstreamPool {
    /// Discovered upstream state, keyed by upstream name.
    catalog: Arc<RwLock<HashMap<String, UpstreamEntry>>>,
    /// Live client connections, keyed by upstream name.
    /// Each is an `Arc<Peer<RoleClient>>` that can `call_tool` / `list_tools`.
    connections: Arc<RwLock<HashMap<String, UpstreamConnection>>>,
    /// Names of upstreams that have `proxy_resources=true`.
    resource_upstreams: Arc<RwLock<Vec<String>>>,
}

/// A live connection to an upstream MCP server.
struct UpstreamConnection {
    /// The running service handle — kept alive to maintain the connection.
    _service: rmcp::service::RunningService<RoleClient, ()>,
    /// The peer handle for making requests.
    peer: rmcp::service::Peer<RoleClient>,
}

impl std::fmt::Debug for UpstreamConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpstreamConnection").finish_non_exhaustive()
    }
}

impl UpstreamPool {
    /// Create a new empty pool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            catalog: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            resource_upstreams: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Connect to all configured upstreams in parallel and discover their tools.
    ///
    /// Each upstream gets a 15-second timeout. Failures are logged and the
    /// upstream is marked unhealthy, but do not prevent other upstreams from
    /// connecting.
    #[allow(clippy::too_many_lines)]
    pub async fn discover_all(&self, configs: &[UpstreamConfig]) {
        if configs.is_empty() {
            return;
        }

        // Validate name uniqueness and URI-safety before starting discovery.
        let mut seen_names = std::collections::HashSet::new();
        for config in configs {
            if !seen_names.insert(&config.name) {
                tracing::warn!(
                    upstream = %config.name,
                    "duplicate upstream name — skipping all but the first"
                );
            }
            if config.name.contains('/') || config.name.contains('?') || config.name.contains('#') {
                tracing::warn!(
                    upstream = %config.name,
                    "upstream name contains URI-unsafe characters (/, ?, #) — skipping"
                );
            }
        }

        // Track which upstreams have resource proxying enabled.
        let resource_names: Vec<String> = configs
            .iter()
            .filter(|c| c.proxy_resources)
            .map(|c| c.name.clone())
            .collect();
        *self.resource_upstreams.write().await = resource_names;

        let mut futures = FuturesUnordered::new();
        let mut processed_names = std::collections::HashSet::new();

        for config in configs {
            // Skip duplicates (only process the first occurrence of each name).
            if !processed_names.insert(&config.name) {
                continue;
            }
            // Skip names with URI-unsafe characters.
            if config.name.contains('/') || config.name.contains('?') || config.name.contains('#') {
                continue;
            }
            // Validate config
            if let Err(msg) = validate_upstream_config(config) {
                tracing::warn!(
                    upstream = %config.name,
                    "skipping upstream: {msg}"
                );
                continue;
            }

            let config = config.clone();
            futures.push(async move {
                let name = config.name.clone();
                match tokio::time::timeout(DISCOVERY_TIMEOUT, connect_upstream(&config)).await {
                    Ok(Ok((conn, tools))) => {
                        tracing::info!(
                            upstream = %name,
                            tool_count = tools.len(),
                            "upstream discovery succeeded"
                        );
                        Ok((name, conn, tools))
                    }
                    Ok(Err(e)) => {
                        tracing::warn!(
                            upstream = %name,
                            error = %e,
                            "upstream discovery failed"
                        );
                        Err(name)
                    }
                    Err(_) => {
                        tracing::warn!(
                            upstream = %name,
                            timeout_secs = DISCOVERY_TIMEOUT.as_secs(),
                            "upstream discovery timed out"
                        );
                        Err(name)
                    }
                }
            });
        }

        // Track all tool names across upstreams to detect duplicates.
        let mut global_tool_names: HashMap<String, String> = HashMap::new();

        while let Some(result) = futures.next().await {
            match result {
                Ok((name, conn, tools)) => {
                    let upstream_name: Arc<str> = Arc::from(name.as_str());
                    let mut tool_map: HashMap<String, UpstreamTool> = HashMap::new();
                    for tool in tools {
                        let schema = if tool.input_schema.is_empty() {
                            None
                        } else {
                            Some(Value::Object((*tool.input_schema).clone()))
                        };
                        let tool_name = tool.name.to_string();
                        // Reject duplicate tool names across upstreams.
                        if let Some(existing_upstream) = global_tool_names.get(&tool_name) {
                            tracing::warn!(
                                tool = %tool_name,
                                upstream = %name,
                                existing_upstream = %existing_upstream,
                                "duplicate tool name across upstreams — skipping"
                            );
                            continue;
                        }
                        global_tool_names.insert(tool_name.clone(), name.clone());
                        tool_map.insert(
                            tool_name,
                            UpstreamTool {
                                tool,
                                input_schema: schema,
                                upstream_name: Arc::clone(&upstream_name),
                            },
                        );
                    }

                    let entry = UpstreamEntry {
                        name: Arc::clone(&upstream_name),
                        tools: tool_map,
                        exposure_policy: ToolExposurePolicy::from_optional(
                            config.expose_tools.clone(),
                        )
                        .unwrap_or(ToolExposurePolicy::All),
                        tool_health: UpstreamHealth::Healthy,
                        prompt_health: UpstreamHealth::Healthy,
                        resource_health: UpstreamHealth::Healthy,
                        tool_unhealthy_since: None,
                        prompt_unhealthy_since: None,
                        resource_unhealthy_since: None,
                    };

                    self.catalog.write().await.insert(name.clone(), entry);
                    self.connections.write().await.insert(name, conn);
                }
                Err(name) => {
                    let entry = UpstreamEntry {
                        name: Arc::from(name.as_str()),
                        tools: HashMap::new(),
                        exposure_policy: ToolExposurePolicy::All,
                        tool_health: UpstreamHealth::Unhealthy {
                            consecutive_failures: 1,
                        },
                        prompt_health: UpstreamHealth::Unhealthy {
                            consecutive_failures: 1,
                        },
                        resource_health: UpstreamHealth::Unhealthy {
                            consecutive_failures: 1,
                        },
                        tool_unhealthy_since: Some(std::time::Instant::now()),
                        prompt_unhealthy_since: Some(std::time::Instant::now()),
                        resource_unhealthy_since: Some(std::time::Instant::now()),
                    };
                    self.catalog.write().await.insert(name, entry);
                }
            }
        }
    }

    /// Get all healthy upstream tools.
    pub async fn healthy_tools(&self) -> Vec<UpstreamTool> {
        let catalog = self.catalog.read().await;
        catalog
            .values()
            .filter(|entry| entry.tool_health.is_routable())
            .flat_map(|entry| {
                entry.tools.values().filter_map(|tool| {
                    entry
                        .exposure_policy
                        .matches(tool.tool.name.as_ref())
                        .then(|| tool.clone())
                })
            })
            .collect()
    }

    /// Return the names of upstreams currently routable for a capability.
    pub async fn routable_upstream_names(&self, capability: UpstreamCapability) -> Vec<String> {
        let catalog = self.catalog.read().await;
        let mut names: Vec<String> = match capability {
            UpstreamCapability::Resources => {
                let resource_names = self.resource_upstreams.read().await;
                resource_names
                    .iter()
                    .filter(|name| {
                        catalog
                            .get(*name)
                            .is_some_and(|entry| entry.health_for(capability).is_routable())
                    })
                    .cloned()
                    .collect()
            }
            UpstreamCapability::Tools | UpstreamCapability::Prompts => catalog
                .iter()
                .filter(|(_, entry)| entry.health_for(capability).is_routable())
                .map(|(name, _)| name.clone())
                .collect(),
        };
        names.sort_unstable();
        names.dedup();
        names
    }

    /// Look up which upstream owns a given tool name.
    #[allow(clippy::significant_drop_tightening)]
    pub async fn find_tool(&self, tool_name: &str) -> Option<(String, UpstreamTool)> {
        let catalog = self.catalog.read().await;
        catalog
            .values()
            .filter(|entry| entry.tool_health.is_routable())
            .find_map(|entry| {
                entry
                    .tools
                    .get(tool_name)
                    .and_then(|tool| {
                        entry
                            .exposure_policy
                            .matches(tool_name)
                            .then(|| (entry.name.to_string(), tool.clone()))
                    })
            })
    }

    /// Get the cached schema for a specific upstream tool.
    #[allow(clippy::significant_drop_tightening)]
    pub async fn tool_schema(&self, tool_name: &str) -> Option<Value> {
        let catalog = self.catalog.read().await;
        catalog.values().find_map(|entry| {
            entry.tools.get(tool_name).and_then(|tool| {
                entry
                    .exposure_policy
                    .matches(tool_name)
                    .then(|| tool.input_schema.clone())
                    .flatten()
            })
        })
    }

    /// Return all discovered tools for one upstream, including hidden tools and exposure metadata.
    pub async fn tool_exposure_rows(&self, upstream_name: &str) -> Vec<UpstreamToolExposureRow> {
        let catalog = self.catalog.read().await;
        let Some(entry) = catalog.get(upstream_name) else {
            return Vec::new();
        };

        let mut rows: Vec<UpstreamToolExposureRow> = entry
            .tools
            .values()
            .map(|tool| {
                let matched_by = entry.exposure_policy.matched_by(tool.tool.name.as_ref());
                UpstreamToolExposureRow {
                    name: tool.tool.name.to_string(),
                    description: Some(tool.tool.description.to_string())
                        .filter(|text| !text.trim().is_empty()),
                    exposed: matched_by.is_some(),
                    matched_by,
                }
            })
            .collect();
        rows.sort_by(|left, right| left.name.cmp(&right.name));
        rows
    }

    /// Return the current tool health for one upstream.
    pub async fn upstream_tool_health(&self, upstream_name: &str) -> Option<UpstreamHealth> {
        let catalog = self.catalog.read().await;
        catalog.get(upstream_name).map(|entry| entry.tool_health)
    }

    /// Call a tool on an upstream server.
    ///
    /// Returns `None` if the upstream is not connected or the tool is not found.
    /// Enforces a response size cap (`LAB_UPSTREAM_MAX_RESPONSE_BYTES`, default 10 MB).
    ///
    /// NOTE: The size check is post-hoc — rmcp materializes the full response before
    /// we can inspect it. This guards against forwarding oversized payloads to callers
    /// but cannot prevent the memory allocation itself. A streaming limit would require
    /// rmcp transport-level support.
    pub async fn call_tool(
        &self,
        upstream_name: &str,
        params: CallToolRequestParams,
    ) -> Option<Result<CallToolResult, String>> {
        let peer = {
            let connections = self.connections.read().await;
            connections.get(upstream_name)?.peer.clone()
        };
        let result = peer
            .call_tool(params)
            .await
            .map_err(|e| format!("upstream call failed: {e}"));

        // Enforce response size cap.
        if let Ok(ref r) = result {
            let response_size = estimate_response_size(r);
            let max_bytes = max_response_bytes();
            if response_size > max_bytes {
                return Some(Err(format!(
                    "upstream response too large ({response_size} bytes, max {max_bytes})"
                )));
            }
        }

        Some(result)
    }

    /// Record a failure for an upstream, potentially marking it unhealthy.
    ///
    /// After [`CIRCUIT_BREAKER_THRESHOLD`] consecutive failures, the upstream
    /// is excluded from `list_tools` until a successful re-probe.
    pub async fn record_failure(&self, upstream_name: &str) {
        self.record_failure_for(upstream_name, UpstreamCapability::Tools)
            .await;
    }

    /// Record a failure for a specific upstream capability, potentially marking it unhealthy.
    ///
    /// After [`CIRCUIT_BREAKER_THRESHOLD`] consecutive failures, the upstream
    /// is excluded from the matching capability listing until a successful re-probe.
    pub async fn record_failure_for(&self, upstream_name: &str, capability: UpstreamCapability) {
        let mut catalog = self.catalog.write().await;
        if let Some(entry) = catalog.get_mut(upstream_name) {
            let new_count = match entry.health_for(capability) {
                UpstreamHealth::Healthy => 1,
                UpstreamHealth::Unhealthy {
                    consecutive_failures,
                } => consecutive_failures + 1,
            };
            entry.set_health_for(
                capability,
                UpstreamHealth::Unhealthy {
                    consecutive_failures: new_count,
                },
            );
            if entry.unhealthy_since_for(capability).is_none() {
                entry.set_unhealthy_since_for(capability, Some(std::time::Instant::now()));
            }
            if new_count >= types::CIRCUIT_BREAKER_THRESHOLD {
                tracing::warn!(
                    upstream = %upstream_name,
                    capability = ?capability,
                    consecutive_failures = new_count,
                    "circuit breaker open — upstream excluded from capability listing"
                );
            }
        }
    }

    /// Record a success for an upstream capability, resetting the circuit breaker.
    pub async fn record_success(&self, upstream_name: &str) {
        self.record_success_for(upstream_name, UpstreamCapability::Tools)
            .await;
    }

    /// Record a success for a specific upstream capability, resetting the circuit breaker.
    pub async fn record_success_for(&self, upstream_name: &str, capability: UpstreamCapability) {
        let mut catalog = self.catalog.write().await;
        if let Some(entry) = catalog.get_mut(upstream_name) {
            if !entry.health_for(capability).is_routable() {
                tracing::info!(
                    upstream = %upstream_name,
                    capability = ?capability,
                    "circuit breaker reset — upstream healthy"
                );
            }
            entry.set_health_for(capability, UpstreamHealth::Healthy);
            entry.set_unhealthy_since_for(capability, None);
        }
    }

    /// Check if an upstream capability is due for a re-probe.
    #[allow(clippy::significant_drop_tightening)]
    pub async fn should_reprobe(&self, upstream_name: &str) -> bool {
        self.should_reprobe_for(upstream_name, UpstreamCapability::Tools)
            .await
    }

    /// Check if a specific upstream capability is due for a re-probe.
    #[allow(clippy::significant_drop_tightening)]
    pub async fn should_reprobe_for(
        &self,
        upstream_name: &str,
        capability: UpstreamCapability,
    ) -> bool {
        let catalog = self.catalog.read().await;
        if let Some(entry) = catalog.get(upstream_name)
            && entry.health_for(capability).is_open()
            && let Some(since) = entry.unhealthy_since_for(capability)
        {
            return since.elapsed() >= types::REPROBE_INTERVAL;
        }
        false
    }

    /// Filter out upstream tools whose names collide with built-in service tools.
    ///
    /// Built-in lab services permanently take precedence. Upstream tools with
    /// colliding names are dropped with a warning.
    pub async fn filter_collisions(&self, builtin_names: &[&str]) {
        let mut catalog = self.catalog.write().await;
        for entry in catalog.values_mut() {
            let collisions: Vec<String> = entry
                .tools
                .keys()
                .filter(|name| builtin_names.contains(&name.as_str()))
                .cloned()
                .collect();
            for name in &collisions {
                tracing::warn!(
                    upstream = %entry.name,
                    tool = %name,
                    "upstream tool name collides with built-in service — rejecting upstream tool"
                );
                entry.tools.remove(name);
            }
        }
    }

    /// Get the number of connected upstreams.
    pub async fn upstream_count(&self) -> usize {
        self.catalog.read().await.len()
    }

    /// Get names of all registered upstreams with their tool health status.
    pub async fn upstream_status(&self) -> Vec<(String, UpstreamHealth)> {
        let catalog = self.catalog.read().await;
        catalog
            .values()
            .map(|e| (e.name.to_string(), e.tool_health))
            .collect()
    }

    /// List resources from all resource-proxy-enabled upstreams.
    ///
    /// Resources are prefixed with `lab://upstream/{name}/` to avoid collisions.
    pub async fn list_upstream_resources(&self) -> Vec<rmcp::model::Resource> {
        let peers = routable_upstream_peers(self, UpstreamCapability::Resources).await;
        if peers.is_empty() {
            return Vec::new();
        }

        let mut resources = Vec::new();
        for (name, peer) in &peers {
            match peer.list_resources(None).await {
                Ok(result) => {
                    self.record_success_for(name, UpstreamCapability::Resources)
                        .await;
                    for mut resource in result.resources {
                        let original_uri = resource.uri.clone();
                        resource.uri = format!("lab://upstream/{name}/{original_uri}");
                        resources.push(resource);
                    }
                }
                Err(e) => {
                    self.record_failure_for(name, UpstreamCapability::Resources)
                        .await;
                    tracing::warn!(
                        upstream = %name,
                        error = %e,
                        "failed to list resources from upstream"
                    );
                }
            }
        }

        resources
    }

    /// Read a resource from an upstream, given a prefixed URI.
    ///
    /// Expects URIs in the form `lab://upstream/{name}/{original_uri}`.
    /// Returns `None` if the upstream name is not found or not resource-enabled.
    pub async fn read_upstream_resource(
        &self,
        uri: &str,
    ) -> Option<Result<rmcp::model::ReadResourceResult, String>> {
        let prefix = "lab://upstream/";
        let rest = uri.strip_prefix(prefix)?;

        // Extract upstream name and original URI
        let slash_pos = rest.find('/')?;
        let upstream_name = &rest[..slash_pos];
        let original_uri = &rest[slash_pos + 1..];

        // Check if this upstream has resource proxying enabled.
        // Clone the vec and drop the lock before any async work.
        let is_resource_enabled = {
            let resource_names = self.resource_upstreams.read().await;
            if !resource_names.iter().any(|n| n == upstream_name) {
                false
            } else {
                let catalog = self.catalog.read().await;
                catalog
                    .get(upstream_name)
                    .is_some_and(|entry| entry.resource_health.is_routable())
            }
        };
        if !is_resource_enabled {
            return None;
        }

        // Clone the peer handle out, then drop the lock before awaiting.
        let peer = {
            let connections = self.connections.read().await;
            connections.get(upstream_name)?.peer.clone()
        };

        let params = rmcp::model::ReadResourceRequestParams::new(original_uri);

        let result = match peer.read_resource(params).await {
            Ok(result) => {
                self.record_success_for(upstream_name, UpstreamCapability::Resources)
                    .await;
                Ok(normalize_resource_result_uri(result, uri))
            }
            Err(e) => {
                self.record_failure_for(upstream_name, UpstreamCapability::Resources)
                    .await;
                Err(format!("upstream resource read failed: {e}"))
            }
        };

        // Enforce the same response size cap as call_tool (post-hoc).
        if let Ok(ref r) = result {
            let response_size = serde_json::to_string(r).map_or(0, |s| s.len());
            let max_bytes = max_response_bytes();
            if response_size > max_bytes {
                return Some(Err(format!(
                    "upstream resource response too large ({response_size} bytes, max {max_bytes})"
                )));
            }
        }

        Some(result)
    }

    /// List prompts from all healthy upstreams, filtering built-in and cross-upstream collisions.
    pub async fn list_upstream_prompts(&self, builtin_names: &[&str]) -> Vec<rmcp::model::Prompt> {
        let peers = routable_upstream_peers(self, UpstreamCapability::Prompts).await;

        let mut upstream_prompts = Vec::new();
        for (name, peer) in &peers {
            match peer.list_prompts(None).await {
                Ok(result) => {
                    self.record_success_for(name, UpstreamCapability::Prompts)
                        .await;
                    upstream_prompts.push((name.clone(), result.prompts));
                }
                Err(e) => {
                    self.record_failure_for(name, UpstreamCapability::Prompts)
                        .await;
                    tracing::warn!(
                        upstream = %name,
                        error = %e,
                        "failed to list prompts from upstream"
                    );
                }
            }
        }

        let (prompts, _) = merge_upstream_prompts(builtin_names, upstream_prompts);
        prompts
    }

    /// Resolve which upstream owns a given prompt name.
    pub async fn find_prompt_owner(&self, prompt_name: &str) -> Option<String> {
        let peers = routable_upstream_peers(self, UpstreamCapability::Prompts).await;

        let mut upstream_prompts = Vec::new();
        for (name, peer) in &peers {
            match peer.list_prompts(None).await {
                Ok(result) => {
                    self.record_success_for(name, UpstreamCapability::Prompts)
                        .await;
                    upstream_prompts.push((name.clone(), result.prompts));
                }
                Err(e) => {
                    self.record_failure_for(name, UpstreamCapability::Prompts)
                        .await;
                    tracing::warn!(
                        upstream = %name,
                        error = %e,
                        "failed to resolve upstream prompt ownership"
                    );
                }
            }
        }

        let (_, owners) = merge_upstream_prompts(&[], upstream_prompts);
        owners.get(prompt_name).cloned()
    }

    /// Proxy a get-prompt request to a specific upstream.
    pub async fn get_prompt(
        &self,
        upstream_name: &str,
        params: GetPromptRequestParams,
    ) -> Option<Result<GetPromptResult, String>> {
        let peer = {
            let connections = self.connections.read().await;
            connections.get(upstream_name)?.peer.clone()
        };

        match peer.get_prompt(params).await {
            Ok(result) => {
                self.record_success_for(upstream_name, UpstreamCapability::Prompts)
                    .await;
                Some(Ok(result))
            }
            Err(e) => {
                self.record_failure_for(upstream_name, UpstreamCapability::Prompts)
                    .await;
                Some(Err(format!("upstream prompt get failed: {e}")))
            }
        }
    }
}

impl Default for UpstreamPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Estimate the serialized size of a `CallToolResult`.
///
/// Uses `serde_json::to_string` as a reasonable approximation. Not exact
/// (ignores transport framing) but sufficient for the size cap guard.
fn estimate_response_size(result: &CallToolResult) -> usize {
    serde_json::to_string(result).map_or(0, |s| s.len())
}

/// Validate an upstream config entry.
fn validate_upstream_config(config: &UpstreamConfig) -> Result<(), String> {
    if config.name.is_empty() {
        return Err("upstream name cannot be empty".into());
    }

    if config.url.is_some() && config.command.is_some() {
        return Err("upstream must not set both 'url' and 'command'".into());
    }

    // Must have either a URL or a command
    if config.url.is_none() && config.command.is_none() {
        return Err("upstream must have either 'url' or 'command'".into());
    }

    if let Some(ref url_str) = config.url {
        // Reject non-HTTP schemes
        if !url_str.starts_with("http://") && !url_str.starts_with("https://") {
            return Err(format!(
                "upstream URL must use http:// or https:// scheme, got: {url_str}"
            ));
        }
        // Parse with url::Url to reliably check the host.
        let parsed = url::Url::parse(url_str)
            .map_err(|e| format!("invalid upstream URL `{url_str}`: {e}"))?;
        if let Some(host) = parsed.host_str() {
            // Reject bind-all addresses (0.0.0.0 or ::).
            let normalized = host.trim_start_matches('[').trim_end_matches(']');
            if normalized == "0.0.0.0" || normalized == "::" {
                return Err("upstream URL must not use 0.0.0.0 or :: (bind-all addresses)".into());
            }
        }
    }

    Ok(())
}

/// Connect to a single upstream MCP server and discover its tools.
async fn connect_upstream(
    config: &UpstreamConfig,
) -> anyhow::Result<(UpstreamConnection, Vec<rmcp::model::Tool>)> {
    if let Some(ref url) = config.url {
        connect_http_upstream(url, config).await
    } else if let Some(ref command) = config.command {
        connect_stdio_upstream(command, &config.args, config).await
    } else {
        anyhow::bail!("upstream {} has neither url nor command", config.name)
    }
}

/// Connect to an HTTP upstream MCP server.
async fn connect_http_upstream(
    url: &str,
    config: &UpstreamConfig,
) -> anyhow::Result<(UpstreamConnection, Vec<rmcp::model::Tool>)> {
    let mut transport_config = StreamableHttpClientTransportConfig::with_uri(url);

    // Set bearer token from env var if configured
    if let Some(ref env_name) = config.bearer_token_env {
        if let Ok(token) = std::env::var(env_name) {
            if !token.is_empty() {
                transport_config.auth_header = Some(token);
            }
        } else {
            tracing::warn!(
                upstream = %config.name,
                env_var = %env_name,
                "bearer_token_env configured but env var not set"
            );
        }
    }

    let worker = StreamableHttpClientWorker::new(reqwest::Client::new(), transport_config);
    let service: rmcp::service::RunningService<RoleClient, ()> = ().serve(worker).await?;
    let peer = service.peer().clone();

    // Discover tools
    let tools = peer.list_all_tools().await?;

    let conn = UpstreamConnection {
        _service: service,
        peer,
    };

    Ok((conn, tools))
}

/// Connect to a stdio upstream MCP server (child process).
async fn connect_stdio_upstream(
    command: &str,
    args: &[String],
    config: &UpstreamConfig,
) -> anyhow::Result<(UpstreamConnection, Vec<rmcp::model::Tool>)> {
    use rmcp::transport::child_process::TokioChildProcess;
    use tokio::process::Command;

    let mut cmd = Command::new(command);
    cmd.args(args);

    // Set bearer token env var on the child if configured
    if let Some(ref env_name) = config.bearer_token_env
        && let Ok(token) = std::env::var(env_name)
    {
        cmd.env(env_name, &token);
    }

    let process = TokioChildProcess::new(cmd)?;
    let service: rmcp::service::RunningService<RoleClient, ()> = ().serve(process).await?;
    let peer = service.peer().clone();

    // Discover tools
    let tools = peer.list_all_tools().await?;

    let conn = UpstreamConnection {
        _service: service,
        peer,
    };

    Ok((conn, tools))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_empty_name() {
        let config = UpstreamConfig {
            name: String::new(),
            url: Some("http://localhost:8080".into()),
            bearer_token_env: None,
            command: None,
            args: vec![],
            proxy_resources: false,
            expose_tools: None,
        };
        assert!(validate_upstream_config(&config).is_err());
    }

    #[test]
    fn validate_rejects_non_http_scheme() {
        let config = UpstreamConfig {
            name: "test".into(),
            url: Some("ftp://example.com".into()),
            bearer_token_env: None,
            command: None,
            args: vec![],
            proxy_resources: false,
            expose_tools: None,
        };
        assert!(validate_upstream_config(&config).is_err());
    }

    #[test]
    fn validate_rejects_bind_all_addresses() {
        for url in &["http://0.0.0.0:8080", "http://[::]/mcp", "http://[::]:8080"] {
            let config = UpstreamConfig {
                name: "test".into(),
                url: Some((*url).into()),
                bearer_token_env: None,
                command: None,
                args: vec![],
                proxy_resources: false,
                expose_tools: None,
            };
            assert!(
                validate_upstream_config(&config).is_err(),
                "should reject {url}"
            );
        }
    }

    #[test]
    fn validate_accepts_valid_http_url() {
        let config = UpstreamConfig {
            name: "test".into(),
            url: Some("http://localhost:8080/mcp".into()),
            bearer_token_env: None,
            command: None,
            args: vec![],
            proxy_resources: false,
            expose_tools: None,
        };
        assert!(validate_upstream_config(&config).is_ok());
    }

    #[test]
    fn validate_accepts_stdio_command() {
        let config = UpstreamConfig {
            name: "test".into(),
            url: None,
            bearer_token_env: None,
            command: Some("my-mcp-server".into()),
            args: vec!["--port".into(), "8080".into()],
            proxy_resources: false,
            expose_tools: None,
        };
        assert!(validate_upstream_config(&config).is_ok());
    }

    #[test]
    fn validate_rejects_both_url_and_command() {
        let config = UpstreamConfig {
            name: "test".into(),
            url: Some("http://localhost:8080".into()),
            bearer_token_env: None,
            command: Some("my-mcp-server".into()),
            args: vec![],
            proxy_resources: false,
            expose_tools: None,
        };
        assert!(validate_upstream_config(&config).is_err());
    }

    #[test]
    fn validate_rejects_no_url_or_command() {
        let config = UpstreamConfig {
            name: "test".into(),
            url: None,
            bearer_token_env: None,
            command: None,
            args: vec![],
            proxy_resources: false,
            expose_tools: None,
        };
        assert!(validate_upstream_config(&config).is_err());
    }

    #[test]
    fn merge_upstream_prompts_is_deterministic() {
        let left = rmcp::model::Prompt::new("shared", Some("left"), None);
        let right = rmcp::model::Prompt::new("shared", Some("right"), None);
        let left_only = rmcp::model::Prompt::new("left-only", Some("left-only"), None);
        let right_only = rmcp::model::Prompt::new("right-only", Some("right-only"), None);

        let (prompts, owners) = merge_upstream_prompts(
            &["builtin"],
            vec![
                ("zeta".into(), vec![right.clone(), right_only]),
                ("alpha".into(), vec![left.clone(), left_only]),
            ],
        );

        let names: Vec<_> = prompts.iter().map(|prompt| prompt.name.as_str()).collect();
        assert_eq!(names, vec!["shared", "left-only", "right-only"]);
        assert_eq!(owners.get("shared").map(String::as_str), Some("alpha"));
        assert_eq!(owners.get("left-only").map(String::as_str), Some("alpha"));
        assert_eq!(owners.get("right-only").map(String::as_str), Some("zeta"));
    }

    #[test]
    fn normalize_resource_result_uri_rewrites_all_contents() {
        let result = rmcp::model::ReadResourceResult::new(vec![
            rmcp::model::ResourceContents::text("hello", "http://upstream/resource"),
            rmcp::model::ResourceContents::blob("YWJj", "file:///tmp/upstream"),
        ]);

        let normalized =
            normalize_resource_result_uri(result, "lab://upstream/demo/http://upstream/resource");

        let uris: Vec<_> = normalized
            .contents
            .iter()
            .map(|content| match content {
                rmcp::model::ResourceContents::TextResourceContents { uri, .. }
                | rmcp::model::ResourceContents::BlobResourceContents { uri, .. } => uri.as_str(),
            })
            .collect();

        assert_eq!(
            uris,
            vec![
                "lab://upstream/demo/http://upstream/resource",
                "lab://upstream/demo/http://upstream/resource",
            ]
        );
    }

    #[tokio::test]
    async fn empty_pool_has_no_tools() {
        let pool = UpstreamPool::new();
        assert!(pool.healthy_tools().await.is_empty());
        assert_eq!(pool.upstream_count().await, 0);
    }

    #[tokio::test]
    async fn hidden_upstream_tools_do_not_appear_in_listings() {
        let pool = UpstreamPool::new();
        let upstream_name: Arc<str> = Arc::from("github");
        let mut tools = HashMap::new();
        for name in ["search_repos", "github_create_issue", "delete_repo"] {
            let schema = Arc::new(serde_json::Map::new());
            let tool = rmcp::model::Tool::new(name, format!("{name} description"), schema);
            tools.insert(
                name.to_string(),
                UpstreamTool {
                    tool,
                    input_schema: None,
                    upstream_name: Arc::clone(&upstream_name),
                },
            );
        }
        let entry = UpstreamEntry {
            name: Arc::clone(&upstream_name),
            tools,
            exposure_policy: ToolExposurePolicy::from_patterns(vec![
                "search_repos".to_string(),
                "github_*".to_string(),
            ])
            .expect("policy"),
            tool_health: UpstreamHealth::Healthy,
            prompt_health: UpstreamHealth::Healthy,
            resource_health: UpstreamHealth::Healthy,
            tool_unhealthy_since: None,
            prompt_unhealthy_since: None,
            resource_unhealthy_since: None,
        };

        pool.catalog.write().await.insert("github".to_string(), entry);

        let names: Vec<String> = pool
            .healthy_tools()
            .await
            .into_iter()
            .map(|t| t.tool.name.to_string())
            .collect();
        assert!(names.contains(&"search_repos".to_string()));
        assert!(names.contains(&"github_create_issue".to_string()));
        assert!(!names.contains(&"delete_repo".to_string()));
    }

    #[tokio::test]
    async fn hidden_upstream_tools_cannot_be_called_directly() {
        let pool = UpstreamPool::new();
        let upstream_name: Arc<str> = Arc::from("github");
        let mut tools = HashMap::new();
        for name in ["search_repos", "delete_repo"] {
            let schema = Arc::new(serde_json::Map::new());
            let tool = rmcp::model::Tool::new(name, format!("{name} description"), schema);
            tools.insert(
                name.to_string(),
                UpstreamTool {
                    tool,
                    input_schema: None,
                    upstream_name: Arc::clone(&upstream_name),
                },
            );
        }
        let entry = UpstreamEntry {
            name: Arc::clone(&upstream_name),
            tools,
            exposure_policy: ToolExposurePolicy::from_patterns(vec!["search_repos".into()])
                .expect("policy"),
            tool_health: UpstreamHealth::Healthy,
            prompt_health: UpstreamHealth::Healthy,
            resource_health: UpstreamHealth::Healthy,
            tool_unhealthy_since: None,
            prompt_unhealthy_since: None,
            resource_unhealthy_since: None,
        };

        pool.catalog.write().await.insert("github".to_string(), entry);

        assert!(pool.find_tool("search_repos").await.is_some());
        assert!(pool.find_tool("delete_repo").await.is_none());
    }
}
