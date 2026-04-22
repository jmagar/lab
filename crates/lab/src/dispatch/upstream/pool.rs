//! `UpstreamPool` — manages connections to upstream MCP servers.
//!
//! Connects to configured upstreams via HTTP (`StreamableHttpClientTransport`)
//! or stdio (child process), discovers their tools, and caches schemas.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::StreamExt;
use futures::stream::FuturesUnordered;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, GetPromptRequestParams, GetPromptResult, Prompt,
    ReadResourceResult, Resource, ResourceContents,
};
use rmcp::transport::streamable_http_client::{
    StreamableHttpClientTransportConfig, StreamableHttpClientWorker,
};
use rmcp::{RoleClient, ServiceExt};
use serde_json::Value;
use tokio::sync::RwLock;

use crate::config::UpstreamConfig;
use crate::oauth::upstream::cache::OauthClientCache;

use super::types;
use super::types::{
    ToolExposurePolicy, UpstreamCapability, UpstreamEntry, UpstreamHealth, UpstreamTool,
    UpstreamToolExposureRow,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UpstreamCachedSummary {
    pub discovered_tool_count: usize,
    pub exposed_tool_count: usize,
    pub discovered_resource_count: usize,
    pub exposed_resource_count: usize,
    pub discovered_prompt_count: usize,
    pub exposed_prompt_count: usize,
}

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

fn upstream_transport(config: &UpstreamConfig) -> &'static str {
    if config.url.is_some() {
        "http"
    } else {
        "stdio"
    }
}

/// Strip query strings and fragments from resource URIs before logging.
///
/// SECURITY: Upstream MCP servers may return resource URIs containing pre-signed
/// tokens or OAuth credentials in query parameters. Only scheme+host+path is safe to log.
pub(crate) fn redact_resource_uri_for_logging(uri: &str) -> &str {
    let cut = uri.find('?').or_else(|| uri.find('#')).unwrap_or(uri.len());
    &uri[..cut]
}

fn upstream_target_redacted(config: &UpstreamConfig) -> String {
    // SECURITY: Never log raw URLs — they may contain userinfo credentials.
    // Strip username/password before logging; log the command path for stdio upstreams.
    match &config.url {
        Some(url_str) => {
            if let Ok(mut parsed) = url::Url::parse(url_str) {
                let _ = parsed.set_username("");
                let _ = parsed.set_password(None);
                parsed.to_string()
            } else {
                "<invalid-url>".to_string()
            }
        }
        None => config.command.clone().unwrap_or_else(|| "<missing>".to_string()),
    }
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

/// Returns true if the error represents a capability the upstream simply doesn't support
/// (method not found / not implemented). These are healthy — the upstream just doesn't
/// expose that capability, which is fine.
fn is_capability_unsupported(error: &rmcp::ServiceError) -> bool {
    let msg = error.to_string();
    msg.contains("Method not found")
        || msg.contains("method_not_found")
        || msg.contains("-32601")
        || msg.contains("Not implemented")
}

async fn discover_capability_counts(
    name: &str,
    peer: &rmcp::service::Peer<RoleClient>,
    proxy_resources: bool,
    proxy_prompts: bool,
) -> (
    usize,
    Option<String>,
    UpstreamHealth,
    usize,
    Option<String>,
    UpstreamHealth,
) {
    let (resource_count, resource_error, resource_health) = if proxy_resources {
        match peer.list_resources(None).await {
            Ok(result) => (result.resources.len(), None, UpstreamHealth::Healthy),
            Err(ref error) if is_capability_unsupported(error) => {
                (0, None, UpstreamHealth::Healthy)
            }
            Err(error) => (
                0,
                Some(format!("failed to list resources from upstream: {error}")),
                UpstreamHealth::Unhealthy {
                    consecutive_failures: 1,
                },
            ),
        }
    } else {
        (0, None, UpstreamHealth::Healthy)
    };

    let (prompt_count, prompt_error, prompt_health) = if proxy_prompts {
        match peer.list_prompts(None).await {
            Ok(result) => (result.prompts.len(), None, UpstreamHealth::Healthy),
            Err(ref error) if is_capability_unsupported(error) => {
                (0, None, UpstreamHealth::Healthy)
            }
            Err(error) => (
                0,
                Some(format!("failed to list prompts from upstream: {error}")),
                UpstreamHealth::Unhealthy {
                    consecutive_failures: 1,
                },
            ),
        }
    } else {
        (0, None, UpstreamHealth::Healthy)
    };

    if let Some(error) = &resource_error {
        tracing::warn!(upstream = %name, error = %error, "failed to discover upstream resources");
    }
    if let Some(error) = &prompt_error {
        tracing::warn!(upstream = %name, error = %error, "failed to discover upstream prompts");
    }

    (
        resource_count,
        resource_error,
        resource_health,
        prompt_count,
        prompt_error,
        prompt_health,
    )
}

/// Merge upstream prompts deterministically and return the winning owner for each prompt.
fn merge_upstream_prompts(
    builtin_names: &[&str],
    mut upstream_prompts: Vec<(String, Vec<Prompt>)>,
) -> (Vec<Prompt>, HashMap<String, String>) {
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
    mut result: ReadResourceResult,
    gateway_uri: &str,
) -> ReadResourceResult {
    for content in &mut result.contents {
        match content {
            ResourceContents::TextResourceContents { uri, .. }
            | ResourceContents::BlobResourceContents { uri, .. } => {
                *uri = gateway_uri.to_string();
            }
        }
    }

    result
}

/// Rewrite an upstream resource's URI to the gateway-prefixed form.
///
/// Strips any embedded upstream name from existing `lab://upstream/…` URIs
/// and re-prefixes with the caller's `upstream_name`.
fn rewrite_resource_uri(resource: &mut Resource, upstream_name: &str) {
    let bare_uri = resource
        .uri
        .strip_prefix("lab://upstream/")
        .and_then(|rest| rest.splitn(2, '/').nth(1).or(Some(rest)))
        .unwrap_or(resource.uri.as_str());
    resource.uri = format!("lab://upstream/{upstream_name}/{bare_uri}");
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
    /// Per-upstream OAuth managers, keyed by upstream name.
    /// `None` when the server was started without OAuth support.
    oauth_client_cache: Option<OauthClientCache>,
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
            oauth_client_cache: None,
        }
    }

    /// Attach the per-`(upstream, subject)` OAuth client cache so the pool can
    /// authenticate OAuth-protected upstreams.
    ///
    /// Must be called before `discover_all` for OAuth upstreams to connect successfully.
    #[must_use]
    pub fn with_oauth_client_cache(mut self, cache: OauthClientCache) -> Self {
        self.oauth_client_cache = Some(cache);
        self
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

        // Track which upstreams have resource/prompt proxying enabled.
        let resource_names: Vec<String> = configs
            .iter()
            .filter(|c| c.proxy_resources)
            .map(|c| c.name.clone())
            .collect();
        *self.resource_upstreams.write().await = resource_names;

        let mut futures = FuturesUnordered::new();
        let mut processed_names = std::collections::HashSet::new();
        let oauth_client_cache = self.oauth_client_cache.clone();

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
            let oauth_client_cache = oauth_client_cache.clone();
            futures.push(async move {
                let name = config.name.clone();
                match tokio::time::timeout(
                    DISCOVERY_TIMEOUT,
                    connect_upstream(&config, None, oauth_client_cache.as_ref()),
                )
                .await
                {
                    Ok(Ok((conn, tools))) => {
                        let (
                            resource_count,
                            resource_last_error,
                            resource_health,
                            prompt_count,
                            prompt_last_error,
                            prompt_health,
                        ) = discover_capability_counts(
                            &name,
                            &conn.peer,
                            config.proxy_resources,
                            config.proxy_prompts,
                        )
                        .await;
                        tracing::info!(
                            upstream = %name,
                            transport = upstream_transport(&config),
                            target = %upstream_target_redacted(&config),
                            tool_count = tools.len(),
                            resource_count,
                            prompt_count,
                            "upstream discovery succeeded"
                        );
                        Ok((
                            name,
                            config.expose_tools.clone(),
                            conn,
                            tools,
                            resource_count,
                            resource_last_error,
                            resource_health,
                            prompt_count,
                            prompt_last_error,
                            prompt_health,
                        ))
                    }
                    Ok(Err(e)) => {
                        let error = e.to_string();
                        tracing::warn!(
                            upstream = %name,
                            transport = upstream_transport(&config),
                            target = %upstream_target_redacted(&config),
                            error = %error,
                            "upstream discovery failed"
                        );
                        Err((name, error))
                    }
                    Err(_) => {
                        let error = format!(
                            "upstream discovery timed out after {}s",
                            DISCOVERY_TIMEOUT.as_secs()
                        );
                        tracing::warn!(
                            upstream = %name,
                            transport = upstream_transport(&config),
                            target = %upstream_target_redacted(&config),
                            timeout_secs = DISCOVERY_TIMEOUT.as_secs(),
                            "upstream discovery timed out"
                        );
                        Err((name, error))
                    }
                }
            });
        }

        // Track all tool names across upstreams to detect duplicates.
        let mut global_tool_names: HashMap<String, String> = HashMap::new();

        while let Some(result) = futures.next().await {
            match result {
                Ok((
                    name,
                    expose_tools,
                    conn,
                    tools,
                    resource_count,
                    resource_last_error,
                    resource_health,
                    prompt_count,
                    prompt_last_error,
                    prompt_health,
                )) => {
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

                    let exposure_policy = resolve_exposure_policy(&name, expose_tools);

                    let entry = UpstreamEntry {
                        name: Arc::clone(&upstream_name),
                        tools: tool_map,
                        exposure_policy,
                        prompt_count,
                        resource_count,
                        prompt_names: Vec::new(),
                        resource_uris: Vec::new(),
                        tool_health: UpstreamHealth::Healthy,
                        prompt_health,
                        resource_health,
                        tool_unhealthy_since: None,
                        prompt_unhealthy_since: (!prompt_health.is_routable())
                            .then(Instant::now),
                        resource_unhealthy_since: (!resource_health.is_routable())
                            .then(Instant::now),
                        tool_last_error: None,
                        prompt_last_error,
                        resource_last_error,
                    };

                    self.catalog.write().await.insert(name.clone(), entry);
                    self.connections.write().await.insert(name, conn);
                }
                Err((name, error_message)) => {
                    let entry = UpstreamEntry {
                        name: Arc::from(name.as_str()),
                        tools: HashMap::new(),
                        exposure_policy: ToolExposurePolicy::All,
                        prompt_count: 0,
                        resource_count: 0,
                        prompt_names: Vec::new(),
                        resource_uris: Vec::new(),
                        tool_health: UpstreamHealth::Unhealthy {
                            consecutive_failures: 1,
                        },
                        prompt_health: UpstreamHealth::Unhealthy {
                            consecutive_failures: 1,
                        },
                        resource_health: UpstreamHealth::Unhealthy {
                            consecutive_failures: 1,
                        },
                        tool_unhealthy_since: Some(Instant::now()),
                        prompt_unhealthy_since: Some(Instant::now()),
                        resource_unhealthy_since: Some(Instant::now()),
                        tool_last_error: Some(error_message.clone()),
                        prompt_last_error: Some(error_message.clone()),
                        resource_last_error: Some(error_message),
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

    pub async fn subject_scoped_tools(
        &self,
        configs: &[UpstreamConfig],
        subject: &str,
    ) -> Vec<(String, Vec<rmcp::model::Tool>)> {
        let mut futures = FuturesUnordered::new();
        let oauth_client_cache = self.oauth_client_cache.clone();
        for config in configs.iter().filter(|config| config.oauth.is_some()) {
            let config = config.clone();
            let subject = subject.to_string();
            let oauth_client_cache = oauth_client_cache.clone();
            futures.push(async move {
                let result =
                    connect_upstream(&config, Some(subject.as_str()), oauth_client_cache.as_ref())
                        .await;
                (config.name.clone(), result)
            });
        }

        let mut discovered = Vec::new();
        while let Some((name, result)) = futures.next().await {
            match result {
                Ok((_conn, tools)) => discovered.push((name, tools)),
                Err(error) => {
                    tracing::warn!(
                        upstream = %name,
                        error = %error,
                        "subject-scoped upstream tool discovery failed"
                    );
                }
            }
        }
        discovered
    }

    pub async fn subject_scoped_call_tool(
        &self,
        config: &UpstreamConfig,
        subject: &str,
        params: CallToolRequestParams,
    ) -> Result<CallToolResult, String> {
        let start = Instant::now();
        let tool_name = params.name.to_string();
        tracing::debug!(
            upstream = %config.name,
            capability = "tools",
            operation = "tool.call",
            tool = %tool_name,
            subject_scoped = true,
            transport = upstream_transport(config),
            "upstream.request.start"
        );
        let (conn, _) = match connect_upstream(
            config,
            Some(subject),
            self.oauth_client_cache.as_ref(),
        )
        .await
        {
            Ok(conn) => conn,
            Err(error) => {
                self.record_failure_for(
                    &config.name,
                    UpstreamCapability::Tools,
                    format!("upstream connect failed: {error}"),
                )
                .await;
                let elapsed_ms = start.elapsed().as_millis();
                tracing::warn!(
                    upstream = %config.name,
                    capability = "tools",
                    operation = "tool.call",
                    tool = %tool_name,
                    subject_scoped = true,
                    transport = upstream_transport(config),
                    elapsed_ms,
                    kind = "upstream_connect_error",
                    error = %error,
                    "upstream.request.error"
                );
                return Err(error.to_string());
            }
        };
        match conn.peer.call_tool(params).await {
            Ok(result) => {
                let response_size = estimate_response_size(&result);
                let max_bytes = max_response_bytes();
                if response_size > max_bytes {
                    self.record_failure_for(
                        &config.name,
                        UpstreamCapability::Tools,
                        format!("response too large: {response_size} bytes"),
                    )
                    .await;
                    let elapsed_ms = start.elapsed().as_millis();
                    tracing::warn!(
                        upstream = %config.name,
                        capability = "tools",
                        operation = "tool.call",
                        tool = %tool_name,
                        subject_scoped = true,
                        transport = upstream_transport(config),
                        elapsed_ms,
                        kind = "response_too_large",
                        response_bytes = response_size,
                        max_bytes,
                        "upstream.request.error"
                    );
                    return Err(format!(
                        "upstream response too large ({response_size} bytes, max {max_bytes})"
                    ));
                }
                self.record_success_for(&config.name, UpstreamCapability::Tools)
                    .await;
                let elapsed_ms = start.elapsed().as_millis();
                tracing::info!(
                    upstream = %config.name,
                    capability = "tools",
                    operation = "tool.call",
                    tool = %tool_name,
                    subject_scoped = true,
                    transport = upstream_transport(config),
                    elapsed_ms,
                    response_bytes = response_size,
                    "upstream.request.finish"
                );
                Ok(result)
            }
            Err(error) => {
                self.record_failure_for(
                    &config.name,
                    UpstreamCapability::Tools,
                    format!("upstream call failed: {error}"),
                )
                .await;
                let elapsed_ms = start.elapsed().as_millis();
                tracing::warn!(
                    upstream = %config.name,
                    capability = "tools",
                    operation = "tool.call",
                    tool = %tool_name,
                    subject_scoped = true,
                    transport = upstream_transport(config),
                    elapsed_ms,
                    kind = "upstream_error",
                    error = %error,
                    "upstream.request.error"
                );
                Err(format!("upstream call failed: {error}"))
            }
        }
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
                entry.tools.get(tool_name).and_then(|tool| {
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
                    description: tool
                        .tool
                        .description
                        .as_ref()
                        .map(ToString::to_string)
                        .filter(|text| !text.trim().is_empty()),
                    exposed: matched_by.is_some(),
                    matched_by,
                }
            })
            .collect();
        rows.sort_by(|left, right| left.name.cmp(&right.name));
        rows
    }

    pub async fn cached_upstream_summary(
        &self,
        upstream_name: &str,
    ) -> Option<UpstreamCachedSummary> {
        let catalog = self.catalog.read().await;
        let entry = catalog.get(upstream_name)?;
        let discovered_tool_count = entry.tools.len();
        let exposed_tool_count = entry
            .tools
            .values()
            .filter(|tool| entry.exposure_policy.matches(tool.tool.name.as_ref()))
            .count();
        let discovered_resource_count = entry.resource_count;
        let exposed_resource_count = if entry.resource_health.is_routable() {
            entry.resource_count
        } else {
            0
        };
        let discovered_prompt_count = entry.prompt_count;
        let exposed_prompt_count = if entry.prompt_health.is_routable() {
            entry.prompt_count
        } else {
            0
        };

        Some(UpstreamCachedSummary {
            discovered_tool_count,
            exposed_tool_count,
            discovered_resource_count,
            exposed_resource_count,
            discovered_prompt_count,
            exposed_prompt_count,
        })
    }

    /// Return cached resource URIs keyed by upstream name (used in catalog snapshots).
    pub async fn cached_upstream_resource_uris(&self) -> Vec<(String, Vec<String>)> {
        let catalog = self.catalog.read().await;
        catalog
            .iter()
            .filter(|(_, entry)| !entry.resource_uris.is_empty())
            .map(|(name, entry)| (name.clone(), entry.resource_uris.clone()))
            .collect()
    }

    /// Return cached prompt names from all upstreams, excluding any that clash with builtins.
    pub async fn cached_upstream_prompt_names(&self, builtins: &[&str]) -> Vec<String> {
        let catalog = self.catalog.read().await;
        catalog
            .values()
            .flat_map(|entry| entry.prompt_names.iter().cloned())
            .filter(|name| !builtins.contains(&name.as_str()))
            .collect()
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
        let start = Instant::now();
        let tool_name = params.name.to_string();
        let peer = {
            let connections = self.connections.read().await;
            connections.get(upstream_name)?.peer.clone()
        };
        tracing::debug!(
            upstream = %upstream_name,
            capability = "tools",
            operation = "tool.call",
            tool = %tool_name,
            subject_scoped = false,
            "upstream.request.start"
        );
        let result = peer
            .call_tool(params)
            .await
            .map_err(|e| {
                let elapsed_ms = start.elapsed().as_millis();
                tracing::warn!(
                    upstream = %upstream_name,
                    capability = "tools",
                    operation = "tool.call",
                    tool = %tool_name,
                    subject_scoped = false,
                    elapsed_ms,
                    kind = "upstream_error",
                    error = %e,
                    "upstream.request.error"
                );
                format!("upstream call failed: {e}")
            });

        // Enforce response size cap.
        if let Ok(ref r) = result {
            let response_size = estimate_response_size(r);
            let max_bytes = max_response_bytes();
            if response_size > max_bytes {
                let elapsed_ms = start.elapsed().as_millis();
                tracing::warn!(
                    upstream = %upstream_name,
                    capability = "tools",
                    operation = "tool.call",
                    tool = %tool_name,
                    subject_scoped = false,
                    elapsed_ms,
                    kind = "response_too_large",
                    response_bytes = response_size,
                    max_bytes,
                    "upstream.request.error"
                );
                return Some(Err(format!(
                    "upstream response too large ({response_size} bytes, max {max_bytes})"
                )));
            }
            let elapsed_ms = start.elapsed().as_millis();
            tracing::info!(
                upstream = %upstream_name,
                capability = "tools",
                operation = "tool.call",
                tool = %tool_name,
                subject_scoped = false,
                elapsed_ms,
                response_bytes = response_size,
                "upstream.request.finish"
            );
        }

        Some(result)
    }

    /// Record a failure for an upstream, potentially marking it unhealthy.
    ///
    /// After [`CIRCUIT_BREAKER_THRESHOLD`] consecutive failures, the upstream
    /// is excluded from `list_tools` until a successful re-probe.
    pub async fn record_failure(&self, upstream_name: &str, error: impl Into<String>) {
        self.record_failure_for(upstream_name, UpstreamCapability::Tools, error)
            .await;
    }

    /// Record a failure for a specific upstream capability, potentially marking it unhealthy.
    ///
    /// After [`CIRCUIT_BREAKER_THRESHOLD`] consecutive failures, the upstream
    /// is excluded from the matching capability listing until a successful re-probe.
    pub async fn record_failure_for(
        &self,
        upstream_name: &str,
        capability: UpstreamCapability,
        error: impl Into<String>,
    ) {
        let mut catalog = self.catalog.write().await;
        if let Some(entry) = catalog.get_mut(upstream_name) {
            let error = error.into();
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
                entry.set_unhealthy_since_for(capability, Some(Instant::now()));
            }
            entry.set_last_error_for(capability, Some(error.clone()));
            if new_count >= types::CIRCUIT_BREAKER_THRESHOLD {
                tracing::warn!(
                    upstream = %upstream_name,
                    capability = ?capability,
                    consecutive_failures = new_count,
                    error = %error,
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
            entry.set_last_error_for(capability, None);
        }
    }

    /// Return the most relevant last error for an upstream, if any capability has one.
    pub async fn upstream_last_error(&self, upstream_name: &str) -> Option<String> {
        let catalog = self.catalog.read().await;
        let entry = catalog.get(upstream_name)?;
        entry
            .last_error_for(UpstreamCapability::Tools)
            .or_else(|| entry.last_error_for(UpstreamCapability::Resources))
            .or_else(|| entry.last_error_for(UpstreamCapability::Prompts))
            .map(ToOwned::to_owned)
    }

    /// Return the last tools-capability error for an upstream, if any.
    pub async fn upstream_tool_last_error(&self, upstream_name: &str) -> Option<String> {
        let catalog = self.catalog.read().await;
        let entry = catalog.get(upstream_name)?;
        entry
            .last_error_for(UpstreamCapability::Tools)
            .map(ToOwned::to_owned)
    }

    #[cfg(test)]
    pub async fn insert_entry_for_tests(&self, name: &str, entry: UpstreamEntry) {
        self.catalog.write().await.insert(name.to_string(), entry);
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
    pub async fn list_upstream_resources(&self) -> Vec<Resource> {
        let peers = routable_upstream_peers(self, UpstreamCapability::Resources).await;
        if peers.is_empty() {
            return Vec::new();
        }

        // Issue RPCs in parallel, then sort by upstream name for deterministic order.
        let mut futures = FuturesUnordered::new();
        for (name, peer) in peers {
            futures.push(async move {
                let result = peer.list_resources(None).await;
                (name, result)
            });
        }

        let mut results: Vec<(String, Result<_, _>)> = Vec::new();
        while let Some(item) = futures.next().await {
            results.push(item);
        }
        results.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut resources = Vec::new();
        for (name, result) in results {
            match result {
                Ok(result) => {
                    self.record_success_for(&name, UpstreamCapability::Resources)
                        .await;
                    {
                        let mut catalog = self.catalog.write().await;
                        if let Some(entry) = catalog.get_mut(&name) {
                            entry.resource_count = result.resources.len();
                        }
                    }
                    for mut resource in result.resources {
                        rewrite_resource_uri(&mut resource, &name);
                        resources.push(resource);
                    }
                }
                Err(e) => {
                    self.record_failure_for(
                        &name,
                        UpstreamCapability::Resources,
                        format!("failed to list resources from upstream: {e}"),
                    )
                    .await;
                    {
                        let mut catalog = self.catalog.write().await;
                        if let Some(entry) = catalog.get_mut(&name) {
                            entry.resource_count = 0;
                        }
                    }
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

    pub async fn subject_scoped_resources(
        &self,
        configs: &[UpstreamConfig],
        subject: &str,
    ) -> Vec<Resource> {
        let mut futures = FuturesUnordered::new();
        let oauth_client_cache = self.oauth_client_cache.clone();
        for config in configs
            .iter()
            .filter(|config| config.oauth.is_some() && config.proxy_resources)
        {
            let config = config.clone();
            let subject = subject.to_string();
            let oauth_client_cache = oauth_client_cache.clone();
            futures.push(async move {
                let result =
                    connect_upstream(&config, Some(subject.as_str()), oauth_client_cache.as_ref())
                        .await
                        .map(|(conn, _)| conn);
                (config.name.clone(), result)
            });
        }

        let mut resources = Vec::new();
        while let Some((name, result)) = futures.next().await {
            let Ok(conn) = result else {
                continue;
            };
            match conn.peer.list_resources(None).await {
                Ok(result) => {
                    for mut resource in result.resources {
                        rewrite_resource_uri(&mut resource, &name);
                        resources.push(resource);
                    }
                }
                Err(error) => {
                    tracing::warn!(
                        upstream = %name,
                        error = %error,
                        "subject-scoped upstream resource discovery failed"
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
    ) -> Option<Result<ReadResourceResult, String>> {
        let start = Instant::now();
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

        tracing::debug!(
            upstream = %upstream_name,
            capability = "resources",
            operation = "resource.read",
            resource_uri = redact_resource_uri_for_logging(uri),
            subject_scoped = false,
            "upstream.request.start"
        );

        let params = rmcp::model::ReadResourceRequestParams::new(original_uri);

        let result = match peer.read_resource(params).await {
            Ok(result) => {
                let normalized = normalize_resource_result_uri(result, uri);
                Ok(normalized)
            }
            Err(e) => {
                self.record_failure_for(
                    upstream_name,
                    UpstreamCapability::Resources,
                    format!("upstream resource read failed: {e}"),
                )
                .await;
                tracing::warn!(
                    upstream = %upstream_name,
                    capability = "resources",
                    operation = "resource.read",
                    resource_uri = redact_resource_uri_for_logging(uri),
                    subject_scoped = false,
                    elapsed_ms = start.elapsed().as_millis(),
                    kind = "upstream_error",
                    error = %e,
                    "upstream.request.error"
                );
                Err(format!("upstream resource read failed: {e}"))
            }
        };

        // Enforce the same response size cap as call_tool (post-hoc).
        if let Ok(ref r) = result {
            let response_size = serde_json::to_string(r).map_or(0, |s| s.len());
            let max_bytes = max_response_bytes();
            if response_size > max_bytes {
                self.record_failure_for(
                    upstream_name,
                    UpstreamCapability::Resources,
                    format!(
                        "upstream resource response too large ({response_size} bytes, max {max_bytes})"
                    ),
                )
                .await;
                tracing::warn!(
                    upstream = %upstream_name,
                    capability = "resources",
                    operation = "resource.read",
                    resource_uri = redact_resource_uri_for_logging(uri),
                    subject_scoped = false,
                    elapsed_ms = start.elapsed().as_millis(),
                    kind = "response_too_large",
                    response_bytes = response_size,
                    max_bytes,
                    "upstream.request.error"
                );
                return Some(Err(format!(
                    "upstream resource response too large ({response_size} bytes, max {max_bytes})"
                )));
            }
            self.record_success_for(upstream_name, UpstreamCapability::Resources)
                .await;
            tracing::info!(
                upstream = %upstream_name,
                capability = "resources",
                operation = "resource.read",
                resource_uri = redact_resource_uri_for_logging(uri),
                subject_scoped = false,
                elapsed_ms = start.elapsed().as_millis(),
                response_bytes = response_size,
                "upstream.request.finish"
            );
        }

        Some(result)
    }

    pub async fn subject_scoped_read_resource(
        &self,
        config: &UpstreamConfig,
        subject: &str,
        uri: &str,
    ) -> Result<ReadResourceResult, String> {
        let start = Instant::now();
        let prefix = format!("lab://upstream/{}/", config.name);
        let original_uri = uri
            .strip_prefix(&prefix)
            .ok_or_else(|| "resource uri does not match upstream".to_string())?;
        tracing::debug!(
            upstream = %config.name,
            capability = "resources",
            operation = "resource.read",
            resource_uri = redact_resource_uri_for_logging(uri),
            subject_scoped = true,
            transport = upstream_transport(config),
            "upstream.request.start"
        );
        let (conn, _) = match connect_upstream(
            config,
            Some(subject),
            self.oauth_client_cache.as_ref(),
        )
        .await
        {
            Ok(conn) => conn,
            Err(error) => {
                self.record_failure_for(
                    &config.name,
                    UpstreamCapability::Resources,
                    format!("upstream resource connect failed: {error}"),
                )
                .await;
                tracing::warn!(
                    upstream = %config.name,
                    capability = "resources",
                    operation = "resource.read",
                    resource_uri = redact_resource_uri_for_logging(uri),
                    subject_scoped = true,
                    transport = upstream_transport(config),
                    elapsed_ms = start.elapsed().as_millis(),
                    kind = "upstream_connect_error",
                    error = %error,
                    "upstream.request.error"
                );
                return Err(error.to_string());
            }
        };
        match conn
            .peer
            .read_resource(rmcp::model::ReadResourceRequestParams::new(original_uri))
            .await
        {
            Ok(result) => {
                // Size check before recording success so an oversized response
                // does not advance the circuit breaker's healthy counter.
                let response_size = serde_json::to_string(&result).map_or(0, |s| s.len());
                let max_bytes = max_response_bytes();
                if response_size > max_bytes {
                    self.record_failure_for(
                        &config.name,
                        UpstreamCapability::Resources,
                        format!(
                            "upstream resource response too large ({response_size} bytes, max {max_bytes})"
                        ),
                    )
                    .await;
                    tracing::warn!(
                        upstream = %config.name,
                        capability = "resources",
                        operation = "resource.read",
                        resource_uri = redact_resource_uri_for_logging(uri),
                        subject_scoped = true,
                        transport = upstream_transport(config),
                        elapsed_ms = start.elapsed().as_millis(),
                        kind = "response_too_large",
                        response_bytes = response_size,
                        max_bytes,
                        "upstream.request.error"
                    );
                    return Err(format!(
                        "upstream resource response too large ({response_size} bytes, max {max_bytes})"
                    ));
                }
                self.record_success_for(&config.name, UpstreamCapability::Resources)
                    .await;
                let normalized = normalize_resource_result_uri(result, uri);
                tracing::info!(
                    upstream = %config.name,
                    capability = "resources",
                    operation = "resource.read",
                    resource_uri = redact_resource_uri_for_logging(uri),
                    subject_scoped = true,
                    transport = upstream_transport(config),
                    elapsed_ms = start.elapsed().as_millis(),
                    response_bytes = response_size,
                    "upstream.request.finish"
                );
                Ok(normalized)
            }
            Err(error) => {
                self.record_failure_for(
                    &config.name,
                    UpstreamCapability::Resources,
                    format!("upstream resource read failed: {error}"),
                )
                .await;
                tracing::warn!(
                    upstream = %config.name,
                    capability = "resources",
                    operation = "resource.read",
                    resource_uri = redact_resource_uri_for_logging(uri),
                    subject_scoped = true,
                    transport = upstream_transport(config),
                    elapsed_ms = start.elapsed().as_millis(),
                    kind = "upstream_error",
                    error = %error,
                    "upstream.request.error"
                );
                Err(format!("upstream resource read failed: {error}"))
            }
        }
    }

    /// Fetch prompts from all healthy upstreams and merge them, returning both the
    /// deduplicated prompt list and the ownership map (prompt_name -> upstream_name).
    ///
    /// This is the single RPC pass shared by all prompt-related queries.
    async fn collect_upstream_prompts(
        &self,
        builtin_names: &[&str],
    ) -> (Vec<Prompt>, HashMap<String, String>) {
        let peers = routable_upstream_peers(self, UpstreamCapability::Prompts).await;

        // Issue RPCs in parallel. merge_upstream_prompts sorts internally,
        // so completion order does not affect the final result.
        let mut futures = FuturesUnordered::new();
        for (name, peer) in peers {
            futures.push(async move {
                let result = peer.list_prompts(None).await;
                (name, result)
            });
        }

        let mut upstream_prompts = Vec::new();
        while let Some((name, result)) = futures.next().await {
            match result {
                Ok(result) => {
                    self.record_success_for(&name, UpstreamCapability::Prompts)
                        .await;
                    {
                        let mut catalog = self.catalog.write().await;
                        if let Some(entry) = catalog.get_mut(&name) {
                            entry.prompt_count = result.prompts.len();
                        }
                    }
                    upstream_prompts.push((name, result.prompts));
                }
                Err(e) => {
                    self.record_failure_for(
                        &name,
                        UpstreamCapability::Prompts,
                        format!("failed to list prompts from upstream: {e}"),
                    )
                    .await;
                    {
                        let mut catalog = self.catalog.write().await;
                        if let Some(entry) = catalog.get_mut(&name) {
                            entry.prompt_count = 0;
                        }
                    }
                    tracing::warn!(
                        upstream = %name,
                        error = %e,
                        "failed to list prompts from upstream"
                    );
                }
            }
        }

        merge_upstream_prompts(builtin_names, upstream_prompts)
    }

    /// List prompts from all healthy upstreams, filtering built-in and cross-upstream collisions.
    pub async fn list_upstream_prompts(&self, builtin_names: &[&str]) -> Vec<Prompt> {
        let (prompts, _) = self.collect_upstream_prompts(builtin_names).await;
        prompts
    }

    pub async fn subject_scoped_prompts(
        &self,
        configs: &[UpstreamConfig],
        subject: &str,
        builtin_names: &[&str],
    ) -> Vec<Prompt> {
        let mut futures = FuturesUnordered::new();
        let oauth_client_cache = self.oauth_client_cache.clone();
        for config in configs.iter().filter(|config| config.oauth.is_some()) {
            let config = config.clone();
            let subject = subject.to_string();
            let oauth_client_cache = oauth_client_cache.clone();
            futures.push(async move {
                let result =
                    connect_upstream(&config, Some(subject.as_str()), oauth_client_cache.as_ref())
                        .await
                        .map(|(conn, _)| conn);
                (config.name.clone(), result)
            });
        }

        let mut upstream_prompts = Vec::new();
        while let Some((name, result)) = futures.next().await {
            let Ok(conn) = result else {
                continue;
            };
            match conn.peer.list_prompts(None).await {
                Ok(result) => upstream_prompts.push((name, result.prompts)),
                Err(error) => {
                    tracing::warn!(
                        upstream = %name,
                        error = %error,
                        "subject-scoped upstream prompt discovery failed"
                    );
                }
            }
        }

        let (prompts, _) = merge_upstream_prompts(builtin_names, upstream_prompts);
        prompts
    }

    /// Build prompt ownership map: prompt_name -> upstream_name.
    ///
    /// Makes M RPCs (one per healthy upstream), not M*N. Use this when you need
    /// to look up ownership for multiple prompts.
    pub async fn prompt_ownership_map(&self, builtin_names: &[&str]) -> HashMap<String, String> {
        let (_, owners) = self.collect_upstream_prompts(builtin_names).await;
        owners
    }

    /// Resolve which upstream owns a given prompt name.
    ///
    /// Prefer `prompt_ownership_map()` when resolving ownership for multiple
    /// prompts to avoid an N+1 RPC pattern.
    pub async fn find_prompt_owner(&self, prompt_name: &str) -> Option<String> {
        let (_, owners) = self.collect_upstream_prompts(&[]).await;
        owners.get(prompt_name).cloned()
    }

    pub async fn subject_scoped_prompt_owner(
        &self,
        configs: &[UpstreamConfig],
        subject: &str,
        prompt_name: &str,
    ) -> Option<String> {
        let mut futures = FuturesUnordered::new();
        let oauth_client_cache = self.oauth_client_cache.clone();
        for config in configs.iter().filter(|config| config.oauth.is_some()) {
            let config = config.clone();
            let subject = subject.to_string();
            let oauth_client_cache = oauth_client_cache.clone();
            let target_prompt = prompt_name.to_string();
            futures.push(async move {
                let result =
                    connect_upstream(&config, Some(subject.as_str()), oauth_client_cache.as_ref())
                        .await
                        .map(|(conn, _)| conn);
                (config.name.clone(), target_prompt, result)
            });
        }

        while let Some((name, target_prompt, result)) = futures.next().await {
            let Ok(conn) = result else {
                continue;
            };
            if let Ok(result) = conn.peer.list_prompts(None).await
                && result
                    .prompts
                    .iter()
                    .any(|prompt| prompt.name == target_prompt)
            {
                return Some(name);
            }
        }
        None
    }

    /// Proxy a get-prompt request to a specific upstream.
    pub async fn get_prompt(
        &self,
        upstream_name: &str,
        params: GetPromptRequestParams,
    ) -> Option<Result<GetPromptResult, String>> {
        let start = Instant::now();
        let prompt_name = params.name.to_string();
        let peer = {
            let connections = self.connections.read().await;
            connections.get(upstream_name)?.peer.clone()
        };

        tracing::debug!(
            upstream = %upstream_name,
            capability = "prompts",
            operation = "prompt.get",
            prompt = %prompt_name,
            subject_scoped = false,
            "upstream.request.start"
        );

        match peer.get_prompt(params).await {
            Ok(result) => {
                self.record_success_for(upstream_name, UpstreamCapability::Prompts)
                    .await;
                tracing::info!(
                    upstream = %upstream_name,
                    capability = "prompts",
                    operation = "prompt.get",
                    prompt = %prompt_name,
                    subject_scoped = false,
                    elapsed_ms = start.elapsed().as_millis(),
                    "upstream.request.finish"
                );
                Some(Ok(result))
            }
            Err(e) => {
                self.record_failure_for(
                    upstream_name,
                    UpstreamCapability::Prompts,
                    format!("upstream prompt get failed: {e}"),
                )
                .await;
                tracing::warn!(
                    upstream = %upstream_name,
                    capability = "prompts",
                    operation = "prompt.get",
                    prompt = %prompt_name,
                    subject_scoped = false,
                    elapsed_ms = start.elapsed().as_millis(),
                    kind = "upstream_error",
                    error = %e,
                    "upstream.request.error"
                );
                Some(Err(format!("upstream prompt get failed: {e}")))
            }
        }
    }

    pub async fn subject_scoped_get_prompt(
        &self,
        config: &UpstreamConfig,
        subject: &str,
        params: GetPromptRequestParams,
    ) -> Result<GetPromptResult, String> {
        let start = Instant::now();
        let prompt_name = params.name.to_string();
        tracing::debug!(
            upstream = %config.name,
            capability = "prompts",
            operation = "prompt.get",
            prompt = %prompt_name,
            subject_scoped = true,
            transport = upstream_transport(config),
            "upstream.request.start"
        );
        let (conn, _) = match connect_upstream(
            config,
            Some(subject),
            self.oauth_client_cache.as_ref(),
        )
        .await
        {
            Ok(conn) => conn,
            Err(error) => {
                self.record_failure_for(
                    &config.name,
                    UpstreamCapability::Prompts,
                    format!("upstream prompt connect failed: {error}"),
                )
                .await;
                tracing::warn!(
                    upstream = %config.name,
                    capability = "prompts",
                    operation = "prompt.get",
                    prompt = %prompt_name,
                    subject_scoped = true,
                    transport = upstream_transport(config),
                    elapsed_ms = start.elapsed().as_millis(),
                    kind = "upstream_connect_error",
                    error = %error,
                    "upstream.request.error"
                );
                return Err(error.to_string());
            }
        };
        match conn.peer.get_prompt(params).await {
            Ok(result) => {
                self.record_success_for(&config.name, UpstreamCapability::Prompts)
                    .await;
                tracing::info!(
                    upstream = %config.name,
                    capability = "prompts",
                    operation = "prompt.get",
                    prompt = %prompt_name,
                    subject_scoped = true,
                    transport = upstream_transport(config),
                    elapsed_ms = start.elapsed().as_millis(),
                    "upstream.request.finish"
                );
                Ok(result)
            }
            Err(error) => {
                self.record_failure_for(
                    &config.name,
                    UpstreamCapability::Prompts,
                    format!("upstream prompt get failed: {error}"),
                )
                .await;
                tracing::warn!(
                    upstream = %config.name,
                    capability = "prompts",
                    operation = "prompt.get",
                    prompt = %prompt_name,
                    subject_scoped = true,
                    transport = upstream_transport(config),
                    elapsed_ms = start.elapsed().as_millis(),
                    kind = "upstream_error",
                    error = %error,
                    "upstream.request.error"
                );
                Err(format!("upstream prompt get failed: {error}"))
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
    subject: Option<&str>,
    oauth_client_cache: Option<&OauthClientCache>,
) -> anyhow::Result<(UpstreamConnection, Vec<rmcp::model::Tool>)> {
    if let Some(ref url) = config.url {
        connect_http_upstream(url, config, subject, oauth_client_cache).await
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
    subject: Option<&str>,
    oauth_client_cache: Option<&OauthClientCache>,
) -> anyhow::Result<(UpstreamConnection, Vec<rmcp::model::Tool>)> {
    let transport_config = StreamableHttpClientTransportConfig::with_uri(url);

    // OAuth path: when the upstream declares oauth config, build an AuthClient.
    if config.oauth.is_some() {
        let subject = subject.ok_or_else(|| {
            anyhow::anyhow!(
                "upstream {} requires an authenticated subject; discovery must be request-scoped",
                config.name
            )
        })?;
        let cache = oauth_client_cache.ok_or_else(|| {
            anyhow::anyhow!(
                "upstream {} requires OAuth but no auth client cache is registered",
                config.name
            )
        })?;

        let auth_client = cache
            .get_or_build(config, subject)
            .await
            .map_err(|e| anyhow::anyhow!("oauth_required: {e}"))?;

        let worker = StreamableHttpClientWorker::new((*auth_client).clone(), transport_config);
        let service: rmcp::service::RunningService<RoleClient, ()> = ().serve(worker).await?;
        let peer = service.peer().clone();
        let tools = peer.list_all_tools().await?;
        return Ok((
            UpstreamConnection {
                _service: service,
                peer,
            },
            tools,
        ));
    }

    // Non-OAuth path: optionally inject a static bearer token from env.
    let mut transport_config = transport_config;
    if let Some(ref env_name) = config.bearer_token_env {
        if let Ok(token) = std::env::var(env_name) {
            let token = token.trim();
            if !token.is_empty() {
                // rmcp calls `.bearer_auth(value)` which prepends "Bearer "
                // automatically, so store only the raw token.
                let raw = if token
                    .get(..7)
                    .is_some_and(|s| s.eq_ignore_ascii_case("bearer "))
                {
                    token[7..].trim()
                } else {
                    token
                };
                if !raw.is_empty() {
                    transport_config.auth_header = Some(raw.to_string());
                }
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
    let tools = peer.list_all_tools().await?;

    Ok((
        UpstreamConnection {
            _service: service,
            peer,
        },
        tools,
    ))
}

/// Connect to a stdio upstream MCP server (child process).
async fn connect_stdio_upstream(
    command: &str,
    args: &[String],
    config: &UpstreamConfig,
) -> anyhow::Result<(UpstreamConnection, Vec<rmcp::model::Tool>)> {
    use rmcp::transport::child_process::TokioChildProcess;
    use std::process::Stdio;
    use tokio::process::Command;

    let mut cmd = Command::new(command);
    cmd.args(args);

    // Set bearer token env var on the child if configured
    if let Some(ref env_name) = config.bearer_token_env
        && let Ok(token) = std::env::var(env_name)
    {
        cmd.env(env_name, &token);
    }

    let (process, _stderr) = TokioChildProcess::builder(cmd)
        .stderr(Stdio::null())
        .spawn()?;
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

fn resolve_exposure_policy(
    upstream_name: &str,
    expose_tools: Option<Vec<String>>,
) -> ToolExposurePolicy {
    match ToolExposurePolicy::from_optional(expose_tools) {
        Ok(policy) => policy,
        Err(error) => {
            tracing::warn!(
                upstream = %upstream_name,
                error = %error,
                "invalid upstream exposure policy; hiding all upstream tools"
            );
            ToolExposurePolicy::AllowList(Vec::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{UpstreamOauthConfig, UpstreamOauthMode, UpstreamOauthRegistration};

    #[test]
    fn validate_rejects_empty_name() {
        let config = UpstreamConfig {
            name: String::new(),
            url: Some("http://localhost:8080".into()),
            bearer_token_env: None,
            command: None,
            args: vec![],
            proxy_resources: false,
            proxy_prompts: false,
            expose_tools: None,
            oauth: None,
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
            proxy_prompts: false,
            expose_tools: None,
            oauth: None,
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
                proxy_prompts: false,
                expose_tools: None,
                oauth: None,
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
            proxy_prompts: false,
            expose_tools: None,
            oauth: None,
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
            proxy_prompts: false,
            expose_tools: None,
            oauth: None,
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
            proxy_prompts: false,
            expose_tools: None,
            oauth: None,
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
            proxy_prompts: false,
            expose_tools: None,
            oauth: None,
        };
        assert!(validate_upstream_config(&config).is_err());
    }

    fn oauth_http_config() -> UpstreamConfig {
        UpstreamConfig {
            name: "oauth-upstream".into(),
            url: Some("http://127.0.0.1:8080/mcp".into()),
            bearer_token_env: None,
            command: None,
            args: vec![],
            proxy_resources: false,
            proxy_prompts: false,
            expose_tools: None,
            oauth: Some(UpstreamOauthConfig {
                mode: UpstreamOauthMode::AuthorizationCodePkce,
                registration: UpstreamOauthRegistration::Preregistered {
                    client_id: "client-id".into(),
                    client_secret_env: None,
                },
                scopes: None,
            }),
        }
    }

    #[tokio::test]
    async fn subject_scoped_upstream_requires_authenticated_subject_for_oauth_http_connect() {
        let config = oauth_http_config();
        let error = connect_http_upstream(
            config.url.as_deref().expect("url"),
            &config,
            None,
            Some(&OauthClientCache::new(Arc::new(dashmap::DashMap::new()))),
        )
        .await
        .expect_err("missing subject should fail");

        assert!(
            error
                .to_string()
                .contains("requires an authenticated subject")
        );
    }

    #[tokio::test]
    async fn subject_scoped_upstream_requires_registered_cache_for_oauth_http_connect() {
        let config = oauth_http_config();
        let error = connect_http_upstream(
            config.url.as_deref().expect("url"),
            &config,
            Some("alice"),
            None,
        )
        .await
        .expect_err("missing cache should fail");

        assert!(
            error
                .to_string()
                .contains("no auth client cache is registered")
        );
    }

    #[test]
    fn merge_upstream_prompts_is_deterministic() {
        let left = Prompt::new("shared", Some("left"), None);
        let right = Prompt::new("shared", Some("right"), None);
        let left_only = Prompt::new("left-only", Some("left-only"), None);
        let right_only = Prompt::new("right-only", Some("right-only"), None);

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
        let result = ReadResourceResult::new(vec![
            ResourceContents::text("hello", "http://upstream/resource"),
            ResourceContents::blob("YWJj", "file:///tmp/upstream"),
        ]);

        let normalized =
            normalize_resource_result_uri(result, "lab://upstream/demo/http://upstream/resource");

        let uris: Vec<_> = normalized
            .contents
            .iter()
            .map(|content| match content {
                ResourceContents::TextResourceContents { uri, .. }
                | ResourceContents::BlobResourceContents { uri, .. } => uri.as_str(),
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
            prompt_count: 0,
            resource_count: 0,
            prompt_names: Vec::new(),
            resource_uris: Vec::new(),
            tool_health: UpstreamHealth::Healthy,
            prompt_health: UpstreamHealth::Healthy,
            resource_health: UpstreamHealth::Healthy,
            tool_unhealthy_since: None,
            prompt_unhealthy_since: None,
            resource_unhealthy_since: None,
            tool_last_error: None,
            prompt_last_error: None,
            resource_last_error: None,
        };

        pool.catalog
            .write()
            .await
            .insert("github".to_string(), entry);

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
            prompt_count: 0,
            resource_count: 0,
            prompt_names: Vec::new(),
            resource_uris: Vec::new(),
            tool_health: UpstreamHealth::Healthy,
            prompt_health: UpstreamHealth::Healthy,
            resource_health: UpstreamHealth::Healthy,
            tool_unhealthy_since: None,
            prompt_unhealthy_since: None,
            resource_unhealthy_since: None,
            tool_last_error: None,
            prompt_last_error: None,
            resource_last_error: None,
        };

        pool.catalog
            .write()
            .await
            .insert("github".to_string(), entry);

        assert!(pool.find_tool("search_repos").await.is_some());
        assert!(pool.find_tool("delete_repo").await.is_none());
    }

    #[tokio::test]
    async fn upstream_last_error_tracks_capability_failure_details() {
        let pool = UpstreamPool::new();
        let upstream_name: Arc<str> = Arc::from("github");
        let entry = UpstreamEntry {
            name: Arc::clone(&upstream_name),
            tools: HashMap::new(),
            exposure_policy: ToolExposurePolicy::All,
            prompt_count: 0,
            resource_count: 0,
            prompt_names: Vec::new(),
            resource_uris: Vec::new(),
            tool_health: UpstreamHealth::Healthy,
            prompt_health: UpstreamHealth::Healthy,
            resource_health: UpstreamHealth::Healthy,
            tool_unhealthy_since: None,
            prompt_unhealthy_since: None,
            resource_unhealthy_since: None,
            tool_last_error: None,
            prompt_last_error: None,
            resource_last_error: None,
        };

        pool.catalog
            .write()
            .await
            .insert("github".to_string(), entry);

        pool.record_failure_for(
            "github",
            UpstreamCapability::Resources,
            "resource listing returned 401 unauthorized",
        )
        .await;

        assert_eq!(
            pool.upstream_last_error("github").await.as_deref(),
            Some("resource listing returned 401 unauthorized")
        );

        pool.record_success_for("github", UpstreamCapability::Resources)
            .await;
        assert_eq!(pool.upstream_last_error("github").await, None);
    }

    #[tokio::test]
    async fn upstream_tool_last_error_ignores_non_tool_failures() {
        let pool = UpstreamPool::new();
        let upstream_name: Arc<str> = Arc::from("github");
        let entry = UpstreamEntry {
            name: Arc::clone(&upstream_name),
            tools: HashMap::new(),
            exposure_policy: ToolExposurePolicy::All,
            prompt_count: 0,
            resource_count: 0,
            prompt_names: Vec::new(),
            resource_uris: Vec::new(),
            tool_health: UpstreamHealth::Healthy,
            prompt_health: UpstreamHealth::Healthy,
            resource_health: UpstreamHealth::Healthy,
            tool_unhealthy_since: None,
            prompt_unhealthy_since: None,
            resource_unhealthy_since: None,
            tool_last_error: None,
            prompt_last_error: None,
            resource_last_error: None,
        };

        pool.catalog
            .write()
            .await
            .insert("github".to_string(), entry);

        pool.record_failure_for(
            "github",
            UpstreamCapability::Resources,
            "resource listing returned 401 unauthorized",
        )
        .await;
        pool.record_failure_for(
            "github",
            UpstreamCapability::Prompts,
            "prompt listing returned 501 unsupported",
        )
        .await;

        assert_eq!(pool.upstream_tool_last_error("github").await, None);

        pool.record_failure_for(
            "github",
            UpstreamCapability::Tools,
            "tool listing returned 500 internal error",
        )
        .await;

        assert_eq!(
            pool.upstream_tool_last_error("github").await.as_deref(),
            Some("tool listing returned 500 internal error")
        );
    }

    #[test]
    fn invalid_exposure_policy_fails_closed() {
        let policy = resolve_exposure_policy("github", Some(vec!["   ".to_string()]));
        assert_eq!(policy, ToolExposurePolicy::AllowList(Vec::new()));
        assert!(!policy.matches("search_repos"));
    }
}
