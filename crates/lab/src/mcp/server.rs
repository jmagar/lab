//! `LabMcpServer` — the MCP `ServerHandler` implementation.
//!
//! Extracted from `cli/serve.rs` so that both the stdio and HTTP transports
//! can share the same handler logic.

use std::collections::BTreeSet;
use std::sync::Arc;

use rmcp::model::{
    AnnotateAble, CallToolRequestParams, CallToolResult, Content, CreateElicitationRequestParams,
    ElicitationAction, ElicitationSchema, GetPromptRequestParams, GetPromptResult,
    ListPromptsResult, ListResourcesResult, ListToolsResult, PaginatedRequestParams,
    PrimitiveSchema, RawResource, ReadResourceRequestParams, ReadResourceResult, ResourceContents,
    ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::{NotificationContext, Peer, RequestContext};
use rmcp::{ErrorData, RoleServer, ServerHandler};
use serde_json::Value;
use tokio::sync::RwLock;
use tokio::sync::mpsc;

use crate::config::DeviceRole;
use crate::dispatch::gateway::manager::GatewayManager;
use crate::dispatch::gateway::types::GatewayCatalogDiff;
use crate::dispatch::upstream::types::UpstreamCapability;
use crate::mcp::envelope::{build_error, build_error_extra, build_success};
use crate::mcp::error::DispatchError;
use crate::registry::ToolRegistry;

/// MCP-specific peer fanout that forwards catalog-change
/// notifications to all connected `rmcp::Peer<RoleServer>` instances.
///
/// This keeps `rmcp` types out of the dispatch layer while allowing
/// `GatewayManager` to notify peers when the upstream pool changes.
#[derive(Clone, Default)]
pub struct PeerNotifier {
    pub peers: Arc<RwLock<Vec<Peer<RoleServer>>>>,
}

impl PeerNotifier {
    pub async fn run(self, mut rx: mpsc::UnboundedReceiver<GatewayCatalogDiff>) {
        while let Some(diff) = rx.recv().await {
            self.notify_catalog_changes(&diff).await;
        }
    }

    async fn notify_catalog_changes(&self, diff: &GatewayCatalogDiff) {
        let peers = self.peers.read().await.clone();
        let mut alive = Vec::with_capacity(peers.len());
        for peer in &peers {
            let peer = peer.clone();
            let mut ok = true;
            if diff.tools_changed {
                ok = peer.notify_tool_list_changed().await.is_ok();
            }
            if ok && diff.resources_changed {
                ok = peer.notify_resource_list_changed().await.is_ok();
            }
            if ok && diff.prompts_changed {
                ok = peer.notify_prompt_list_changed().await.is_ok();
            }
            if ok {
                alive.push(peer);
            }
        }

        let mut guard = self.peers.write().await;
        let added_since_snapshot = guard.split_off(peers.len());
        *guard = alive;
        guard.extend(added_since_snapshot);
    }
}

/// JSON Schema for every service tool's input: `action` (required) + `params` (optional object).
#[allow(clippy::expect_used)]
fn action_schema() -> serde_json::Map<String, Value> {
    serde_json::json!({
        "type": "object",
        "properties": {
            "action": {
                "type": "string",
                "description": "Action to perform (e.g. \"movie.search\"). Use \"help\" to list all actions."
            },
            "params": {
                "type": "object",
                "description": "Action-specific parameters (varies per action)"
            }
        },
        "required": ["action"]
    })
    .as_object()
    .cloned()
    .expect("schema literal is always an object")
}

/// MCP server handler — one tool per registered service.
pub struct LabMcpServer {
    pub registry: Arc<ToolRegistry>,
    /// Shared gateway manager used to resolve the current live upstream pool.
    pub gateway_manager: Option<Arc<GatewayManager>>,
    /// Resolved role for the current device.
    pub device_role: Option<DeviceRole>,
    /// Connected peers for list-changed notifications.
    pub peers: Arc<RwLock<Vec<Peer<RoleServer>>>>,
}

impl ServerHandler for LabMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_tool_list_changed()
                .enable_resources()
                .enable_resources_list_changed()
                .enable_prompts()
                .enable_prompts_list_changed()
                .build(),
        )
    }

    async fn on_initialized(&self, context: NotificationContext<RoleServer>) {
        self.peers.write().await.push(context.peer);
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, ErrorData> {
        let before = self.snapshot_catalog().await;
        let mut prompts = crate::mcp::prompts::list_all().prompts;

        if let Some(pool) = self.current_upstream_pool().await {
            let builtin_names: Vec<&str> =
                prompts.iter().map(|prompt| prompt.name.as_ref()).collect();
            let upstream_prompts = pool.list_upstream_prompts(&builtin_names).await;
            prompts.extend(upstream_prompts);
        }

        let after = self.snapshot_catalog().await;
        self.notify_catalog_changes(&before, &after).await;

        Ok(ListPromptsResult::with_all_items(prompts))
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, ErrorData> {
        let before = self.snapshot_catalog().await;
        let args = request
            .arguments
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|(key, value)| {
                let string = match value {
                    Value::String(text) => text,
                    other => other.to_string(),
                };
                (key, string)
            })
            .collect();

        if let Some(prompt) = crate::mcp::prompts::get(&self.registry, &request.name, &args) {
            return Ok(prompt);
        }

        if let Some(pool) = self.current_upstream_pool().await
            && let Some(upstream_name) = pool.find_prompt_owner(&request.name).await
        {
            let prompt_name = request.name.clone();
            let outcome = match pool.get_prompt(&upstream_name, request).await {
                Some(Ok(result)) => Ok(result),
                Some(Err(message)) => Err(ErrorData::internal_error(message, None)),
                None => Err(ErrorData::invalid_params(
                    format!("unknown prompt: {prompt_name}"),
                    None,
                )),
            };
            let after = self.snapshot_catalog().await;
            self.notify_catalog_changes(&before, &after).await;
            return outcome;
        }

        Err(ErrorData::invalid_params(
            format!("unknown prompt: {}", request.name),
            None,
        ))
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        let before = self.snapshot_catalog().await;
        let mut resources = vec![
            RawResource::new("lab://catalog", "catalog")
                .with_description("Full discovery document for all services")
                .with_mime_type("application/json")
                .no_annotation(),
        ];

        for svc in self.registry.services() {
            let uri = format!("lab://{}/actions", svc.name);
            let name = format!("{}/actions", svc.name);
            resources.push(
                RawResource::new(uri, name)
                    .with_description(format!("Action list for {}", svc.name))
                    .with_mime_type("application/json")
                    .no_annotation(),
            );
        }

        if let Some(pool) = self.current_upstream_pool().await {
            resources.extend(pool.list_upstream_resources().await);
        }

        let after = self.snapshot_catalog().await;
        self.notify_catalog_changes(&before, &after).await;

        Ok(ListResourcesResult::with_all_items(resources))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        let before = self.snapshot_catalog().await;
        let uri = &request.uri;

        if let Some(pool) = self.current_upstream_pool().await
            && uri.starts_with("lab://upstream/")
        {
            let outcome = match pool.read_upstream_resource(uri).await {
                Some(Ok(result)) => Ok(result),
                Some(Err(message)) => Err(ErrorData::internal_error(message, None)),
                None => Err(ErrorData::resource_not_found(
                    format!("unknown resource: {uri}"),
                    None,
                )),
            };
            let after = self.snapshot_catalog().await;
            self.notify_catalog_changes(&before, &after).await;
            return outcome;
        }

        let json = if uri == "lab://catalog" {
            self.catalog_json().await
        } else if let Some(service) = uri
            .strip_prefix("lab://")
            .and_then(|value| value.strip_suffix("/actions"))
        {
            self.service_actions_json(service).await
        } else {
            return Err(ErrorData::resource_not_found(
                format!("unknown resource: {uri}"),
                None,
            ));
        };

        match json {
            Ok(value) => {
                let text = serde_json::to_string_pretty(&value).unwrap_or_default();
                Ok(ReadResourceResult::new(vec![
                    ResourceContents::text(text, uri.clone()).with_mime_type("application/json"),
                ]))
            }
            Err(e) => Err(ErrorData::internal_error(e.to_string(), None)),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let schema = Arc::new(action_schema());
        let mut tools = Vec::new();
        for svc in self.registry.services() {
            if self.service_visible_on_mcp(svc.name).await {
                tools.push(Tool::new(svc.name, svc.description, Arc::clone(&schema)));
            }
        }

        // Merge upstream tools (healthy only, filtered for collisions with built-in services).
        if let Some(pool) = self.current_upstream_pool().await {
            let mut builtin_names = Vec::new();
            for service in self.registry.services() {
                if self.service_visible_on_mcp(service.name).await {
                    builtin_names.push(service.name);
                }
            }
            let upstream_tools = pool.healthy_tools().await;
            for ut in upstream_tools {
                let tool_name = ut.tool.name.as_ref();
                if builtin_names.contains(&tool_name) {
                    tracing::debug!(
                        tool = tool_name,
                        "skipping upstream tool that collides with built-in service"
                    );
                    continue;
                }
                tools.push(ut.tool);
            }
        }

        Ok(ListToolsResult::with_all_items(tools))
    }

    #[allow(clippy::too_many_lines)]
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let service = request.name.as_ref().to_string();
        let raw_arguments = request.arguments.clone();
        let args = request.arguments.unwrap_or_default();
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let params = args.get("params").cloned().unwrap_or(Value::Null);

        let svc = self.registry.services().iter().find(|s| s.name == service);
        if svc.is_some() && !self.service_visible_on_mcp(&service).await {
            let envelope = build_error(
                &service,
                &action,
                "not_found",
                &format!("service `{service}` is not enabled on the mcp surface"),
            );
            return Ok(CallToolResult::error(vec![Content::text(
                envelope.to_string(),
            )]));
        }

        if svc.is_some() && !self.action_allowed_on_mcp(&service, &action).await {
            let mut extra = serde_json::Map::new();
            if let Some(valid) = self.allowed_mcp_actions(&service).await {
                extra.insert(
                    "valid".to_string(),
                    serde_json::to_value(valid).unwrap_or(Value::Array(Vec::new())),
                );
            }
            let envelope = build_error_extra(
                &service,
                &action,
                "unknown_action",
                &format!("action `{action}` is not exposed for service `{service}`"),
                &Value::Object(extra),
            );
            return Ok(CallToolResult::error(vec![Content::text(
                envelope.to_string(),
            )]));
        }

        // Elicitation gate: if the action is destructive and the client supports
        // elicitation, ask for confirmation before dispatching.
        if let Some(entry) = svc {
            let is_destructive = entry
                .actions
                .iter()
                .any(|a| a.name == action && a.destructive);
            if is_destructive {
                match elicit_confirm(&context, &service, &action).await {
                    ElicitResult::Confirmed => {}
                    ElicitResult::Declined | ElicitResult::Cancelled => {
                        let envelope = build_error(
                            &service,
                            &action,
                            "confirmation_required",
                            &format!("action `{action}` is destructive — confirm to proceed"),
                        );
                        return Ok(CallToolResult::error(vec![Content::text(
                            envelope.to_string(),
                        )]));
                    }
                    ElicitResult::NotSupported => {
                        // Client does not support elicitation — allow params["confirm"] == true
                        // as a machine-to-machine bypass (mirrors HTTP's handle_action()).
                        if params.get("confirm").and_then(Value::as_bool) != Some(true) {
                            let envelope = build_error(
                                &service,
                                &action,
                                "confirmation_required",
                                &format!(
                                    "action `{action}` is destructive — pass \
                                     {{\"confirm\":true}} in params or use a client \
                                     that supports MCP elicitation"
                                ),
                            );
                            return Ok(CallToolResult::error(vec![Content::text(
                                envelope.to_string(),
                            )]));
                        }
                    }
                    ElicitResult::Failed => {
                        let envelope = build_error(
                            &service,
                            &action,
                            "confirmation_required",
                            &format!(
                                "action `{action}` is destructive — confirmation failed, retry with a client that supports MCP elicitation"
                            ),
                        );
                        return Ok(CallToolResult::error(vec![Content::text(
                            envelope.to_string(),
                        )]));
                    }
                }
            }
        }

        let start = std::time::Instant::now();

        // Try built-in dispatch first.
        if let Some(entry) = svc {
            let result = (entry.dispatch)(action.clone(), params)
                .await
                .map_err(|te| anyhow::Error::from(DispatchError::from(te)));
            let elapsed_ms = start.elapsed().as_millis();
            return Ok(format_dispatch_result(
                result, &service, &action, elapsed_ms,
            ));
        }

        // Fall through to upstream proxy dispatch.
        // Upstream tools don't use lab's action/params wrapper — they receive
        // raw arguments. Use "call_tool" as the action label for logging/envelopes.
        let upstream_action = "call_tool";
        if let Some(pool) = self.current_upstream_pool().await
            && let Some((upstream_name, _tool)) = pool.find_tool(&service).await
        {
            let before = self.snapshot_catalog().await;
            tracing::debug!(
                surface = "mcp",
                service,
                action = upstream_action,
                upstream = %upstream_name,
                "proxying to upstream"
            );

            let mut upstream_params = CallToolRequestParams::new(service.clone());
            upstream_params.arguments = raw_arguments;

            match pool.call_tool(&upstream_name, upstream_params).await {
                Some(Ok(result)) => {
                    let elapsed_ms = start.elapsed().as_millis();
                    let (result, kind, counts_as_failure) =
                        normalize_upstream_result(&service, upstream_action, result);
                    if counts_as_failure {
                        pool.record_failure(
                            &upstream_name,
                            format!("upstream `{upstream_name}` returned `{kind}`"),
                        )
                        .await;
                        tracing::warn!(
                            surface = "mcp",
                            service,
                            action = upstream_action,
                            upstream = %upstream_name,
                            elapsed_ms,
                            kind,
                            "upstream proxy failed"
                        );
                    } else {
                        pool.record_success(&upstream_name).await;
                        tracing::info!(
                            surface = "mcp",
                            service,
                            action = upstream_action,
                            upstream = %upstream_name,
                            elapsed_ms,
                            "upstream proxy ok"
                        );
                    }
                    let after = self.snapshot_catalog().await;
                    self.notify_catalog_changes(&before, &after).await;
                    return Ok(result);
                }
                Some(Err(e)) => {
                    pool.record_failure(&upstream_name, e.clone()).await;
                    let after = self.snapshot_catalog().await;
                    self.notify_catalog_changes(&before, &after).await;
                    let elapsed_ms = start.elapsed().as_millis();
                    tracing::warn!(
                        surface = "mcp",
                        service,
                        action = upstream_action,
                        upstream = %upstream_name,
                        elapsed_ms,
                        kind = "upstream_error",
                        "upstream proxy failed"
                    );
                    let envelope = build_error(
                        &service,
                        upstream_action,
                        "upstream_error",
                        &format!("upstream `{upstream_name}` call failed: {e}"),
                    );
                    return Ok(CallToolResult::error(vec![Content::text(
                        envelope.to_string(),
                    )]));
                }
                None => {
                    // Connection is gone — record failure so the circuit
                    // breaker can eventually exclude this upstream.
                    pool.record_failure(
                        &upstream_name,
                        format!("upstream `{upstream_name}` is not connected"),
                    )
                    .await;
                    let after = self.snapshot_catalog().await;
                    self.notify_catalog_changes(&before, &after).await;
                    let elapsed_ms = start.elapsed().as_millis();
                    tracing::warn!(
                        surface = "mcp",
                        service,
                        action = upstream_action,
                        upstream = %upstream_name,
                        elapsed_ms,
                        kind = "upstream_error",
                        "upstream not connected"
                    );
                    let envelope = build_error(
                        &service,
                        upstream_action,
                        "upstream_error",
                        &format!("upstream `{upstream_name}` is not connected"),
                    );
                    return Ok(CallToolResult::error(vec![Content::text(
                        envelope.to_string(),
                    )]));
                }
            }
        }

        // Neither built-in nor upstream.
        let elapsed_ms = start.elapsed().as_millis();
        let err = anyhow::anyhow!("service `{service}` has no dispatcher wired");
        Ok(format_dispatch_result(
            Err(err),
            &service,
            &action,
            elapsed_ms,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CatalogSnapshot {
    tools: BTreeSet<String>,
    resources: BTreeSet<String>,
    prompts: BTreeSet<String>,
}

impl LabMcpServer {
    async fn current_upstream_pool(
        &self,
    ) -> Option<Arc<crate::dispatch::upstream::pool::UpstreamPool>> {
        match &self.gateway_manager {
            Some(manager) => manager.current_pool().await,
            None => None,
        }
    }

    async fn service_visible_on_mcp(&self, service: &str) -> bool {
        if matches!(self.device_role, Some(DeviceRole::NonMaster)) {
            return false;
        }
        match &self.gateway_manager {
            Some(manager) => manager.surface_enabled_for_service(service, "mcp").await,
            None => true,
        }
    }

    async fn action_allowed_on_mcp(&self, service: &str, action: &str) -> bool {
        match &self.gateway_manager {
            Some(manager) => {
                manager
                    .mcp_action_allowed_for_service(service, action)
                    .await
            }
            None => true,
        }
    }

    async fn allowed_mcp_actions(&self, service: &str) -> Option<Vec<String>> {
        match &self.gateway_manager {
            Some(manager) => manager.allowed_mcp_actions_for_service(service).await,
            None => None,
        }
    }

    async fn catalog_json(&self) -> anyhow::Result<Value> {
        let mut catalog = crate::catalog::build_catalog(&self.registry);
        let mut services = Vec::new();
        for mut service in catalog.services {
            if !self.service_visible_on_mcp(&service.name).await {
                continue;
            }
            if let Some(allowed_actions) = self.allowed_mcp_actions(&service.name).await
                && !allowed_actions.is_empty()
            {
                service
                    .actions
                    .retain(|action| allowed_actions.contains(&action.name));
            }
            services.push(service);
        }
        catalog.services = services;
        Ok(serde_json::to_value(catalog)?)
    }

    async fn service_actions_json(&self, service: &str) -> anyhow::Result<Value> {
        if !self.service_visible_on_mcp(service).await {
            anyhow::bail!("unknown service: {service}");
        }

        let catalog = crate::catalog::build_catalog(&self.registry);
        let mut entry = catalog
            .services
            .into_iter()
            .find(|entry| entry.name == service)
            .ok_or_else(|| anyhow::anyhow!("unknown service: {service}"))?;

        if let Some(allowed_actions) = self.allowed_mcp_actions(service).await
            && !allowed_actions.is_empty()
        {
            entry
                .actions
                .retain(|action| allowed_actions.contains(&action.name));
        }

        Ok(serde_json::to_value(entry.actions)?)
    }

    async fn snapshot_catalog(&self) -> CatalogSnapshot {
        let mut tools = BTreeSet::new();
        for svc in self.registry.services() {
            if self.service_visible_on_mcp(svc.name).await {
                tools.insert(svc.name.to_string());
            }
        }

        if let Some(pool) = self.current_upstream_pool().await {
            for tool in pool.healthy_tools().await {
                let name = tool.tool.name.to_string();
                if !tools.contains(&name) {
                    tools.insert(name);
                }
            }
        }

        let resources = if let Some(pool) = self.current_upstream_pool().await {
            pool.routable_upstream_names(UpstreamCapability::Resources)
                .await
                .into_iter()
                .collect()
        } else {
            BTreeSet::new()
        };

        let prompts = if let Some(pool) = self.current_upstream_pool().await {
            pool.routable_upstream_names(UpstreamCapability::Prompts)
                .await
                .into_iter()
                .collect()
        } else {
            BTreeSet::new()
        };

        CatalogSnapshot {
            tools,
            resources,
            prompts,
        }
    }

    async fn notify_catalog_changes(&self, before: &CatalogSnapshot, after: &CatalogSnapshot) {
        if before == after {
            return;
        }

        let peers = self.peers.read().await.clone();
        let peer_count = peers.len();
        let mut alive = Vec::with_capacity(peers.len());
        for peer in peers {
            let mut ok = true;
            if before.tools != after.tools {
                if peer.notify_tool_list_changed().await.is_err() {
                    ok = false;
                }
            }
            if ok && before.resources != after.resources {
                if peer.notify_resource_list_changed().await.is_err() {
                    ok = false;
                }
            }
            if ok && before.prompts != after.prompts {
                if peer.notify_prompt_list_changed().await.is_err() {
                    ok = false;
                }
            }
            if ok {
                alive.push(peer);
            }
        }
        let mut guard = self.peers.write().await;
        let added_since_snapshot = if guard.len() > peer_count {
            guard.split_off(peer_count)
        } else {
            Vec::new()
        };
        *guard = alive;
        guard.extend(added_since_snapshot);
    }
}

/// Format the result of a dispatch operation into an MCP `CallToolResult`.
fn format_dispatch_result(
    result: Result<Value, anyhow::Error>,
    service: &str,
    action: &str,
    elapsed_ms: u128,
) -> CallToolResult {
    match result {
        Ok(v) => {
            tracing::info!(surface = "mcp", service, action, elapsed_ms, "dispatch ok");
            let envelope = build_success(service, action, &v);
            CallToolResult::success(vec![Content::text(envelope.to_string())])
        }
        Err(e) => {
            let (kind, message, extra) = extract_error_info(&e);
            let is_fatal = matches!(kind, "internal_error" | "server_error" | "decode_error");
            if is_fatal {
                tracing::error!(
                    surface = "mcp",
                    service,
                    action,
                    elapsed_ms,
                    kind,
                    "dispatch error"
                );
            } else {
                tracing::warn!(
                    surface = "mcp",
                    service,
                    action,
                    elapsed_ms,
                    kind,
                    "dispatch error"
                );
            }
            let envelope = extra.map_or_else(
                || build_error(service, action, kind, &message),
                |ref extra| build_error_extra(service, action, kind, &message, extra),
            );
            CallToolResult::error(vec![Content::text(envelope.to_string())])
        }
    }
}

/// Outcome of an elicitation confirmation request.
enum ElicitResult {
    /// User confirmed the destructive action.
    Confirmed,
    /// User explicitly declined.
    Declined,
    /// User cancelled (closed the dialog without choosing).
    Cancelled,
    /// MCP client does not support the elicitation capability.
    NotSupported,
    /// The client advertised elicitation support, but the RPC failed.
    Failed,
}

/// Ask the MCP client to confirm a destructive action via elicitation.
///
/// Sends a form with a single required `confirm: boolean` field.
/// Returns `NotSupported` if the client's capabilities do not include elicitation.
async fn elicit_confirm(
    context: &RequestContext<RoleServer>,
    service: &str,
    action: &str,
) -> ElicitResult {
    if context.peer.supported_elicitation_modes().is_empty() {
        return ElicitResult::NotSupported;
    }

    let Ok(schema) = ElicitationSchema::builder()
        .required_property(
            "confirm",
            PrimitiveSchema::Boolean(rmcp::model::BooleanSchema::default()),
        )
        .build()
    else {
        return ElicitResult::NotSupported;
    };

    let params = CreateElicitationRequestParams::FormElicitationParams {
        meta: None,
        message: format!(
            "Action `{service}.{action}` is destructive and cannot be undone. \
             Set `confirm` to true to proceed."
        ),
        requested_schema: schema,
    };

    match context.peer.create_elicitation(params).await {
        Ok(result) => match result.action {
            ElicitationAction::Accept => {
                // Check that the user actually set confirm = true in the response.
                let confirmed = result
                    .content
                    .as_ref()
                    .and_then(|v| v.get("confirm"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                if confirmed {
                    ElicitResult::Confirmed
                } else {
                    ElicitResult::Declined
                }
            }
            ElicitationAction::Decline => ElicitResult::Declined,
            ElicitationAction::Cancel => ElicitResult::Cancelled,
        },
        Err(_) => ElicitResult::Failed,
    }
}

fn normalize_upstream_result(
    service: &str,
    action: &str,
    result: CallToolResult,
) -> (CallToolResult, &'static str, bool) {
    if result.is_error != Some(true) {
        return (result, "ok", false);
    }

    let Some(text) = result
        .content
        .first()
        .and_then(|content| content.as_text())
        .map(|content| content.text.as_str())
    else {
        let envelope = build_error(
            service,
            action,
            "upstream_error",
            "upstream returned a non-text error payload",
        );
        return (
            CallToolResult::error(vec![Content::text(envelope.to_string())]),
            "upstream_error",
            true,
        );
    };

    let Ok(parsed) = serde_json::from_str::<Value>(text) else {
        let envelope = build_error(service, action, "upstream_error", text);
        return (
            CallToolResult::error(vec![Content::text(envelope.to_string())]),
            "upstream_error",
            true,
        );
    };

    let error_obj = parsed
        .get("error")
        .and_then(Value::as_object)
        .or_else(|| parsed.as_object());

    let Some(error_obj) = error_obj else {
        let envelope = build_error(service, action, "upstream_error", text);
        return (
            CallToolResult::error(vec![Content::text(envelope.to_string())]),
            "upstream_error",
            true,
        );
    };

    let kind = error_obj
        .get("kind")
        .and_then(Value::as_str)
        .map(static_kind)
        .unwrap_or("upstream_error");
    let message = error_obj
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or(text);

    let extra = serde_json::Map::from_iter(
        error_obj
            .iter()
            .filter(|(key, _)| *key != "kind" && *key != "message")
            .map(|(key, value)| (key.clone(), value.clone())),
    );

    let envelope = if extra.is_empty() {
        build_error(service, action, kind, message)
    } else {
        build_error_extra(service, action, kind, message, &Value::Object(extra))
    };

    (
        CallToolResult::error(vec![Content::text(envelope.to_string())]),
        kind,
        matches!(
            kind,
            "upstream_error" | "network_error" | "server_error" | "decode_error" | "internal_error"
        ),
    )
}

/// Recover a stable kind tag and message from an `anyhow::Error`.
///
/// Priority:
/// 1. Downcast to [`DispatchError`] — gives structured kind + optional extras.
/// 2. Parse `e.to_string()` as JSON `{ "kind": "…" }` — covers `ToolError`
///    errors that were serialized to string before entering anyhow (radarr).
/// 3. Fall back to `"internal_error"`.
pub fn extract_error_info(e: &anyhow::Error) -> (&'static str, String, Option<Value>) {
    // 1. Structured DispatchError
    if let Some(de) = e.downcast_ref::<DispatchError>() {
        let extra = if de.valid.is_some() || de.param.is_some() || de.hint.is_some() {
            Some(serde_json::json!({
                "valid": de.valid,
                "param": de.param,
                "hint":  de.hint,
            }))
        } else {
            None
        };
        return (de.kind, de.message.clone(), extra);
    }
    // 2. ToolError serialized as JSON string (legacy radarr path)
    let msg = e.to_string();
    if let Ok(v) = serde_json::from_str::<Value>(&msg)
        && let Some(kind_str) = v.get("kind").and_then(|k| k.as_str())
    {
        let kind: &'static str = static_kind(kind_str);
        let message = v["message"].as_str().unwrap_or(&msg).to_string();
        // Preserve structured extras (valid list, param name, hint) if present.
        let has_valid = v.get("valid").is_some_and(|v| !v.is_null());
        let has_param = v.get("param").is_some_and(|v| !v.is_null());
        let has_hint = v.get("hint").is_some_and(|v| !v.is_null());
        let extra = if has_valid || has_param || has_hint {
            Some(serde_json::json!({
                "valid": v.get("valid"),
                "param": v.get("param"),
                "hint":  v.get("hint"),
            }))
        } else {
            None
        };
        return (kind, message, extra);
    }
    // 3. Generic fallback
    ("internal_error", msg, None)
}

/// Map a serialized kind string to a `&'static str` from the canonical vocabulary.
pub fn static_kind(s: &str) -> &'static str {
    match s {
        "unknown_action" => "unknown_action",
        "unknown_subaction" => "unknown_subaction",
        "missing_param" => "missing_param",
        "invalid_param" => "invalid_param",
        "unknown_instance" => "unknown_instance",
        "auth_failed" => "auth_failed",
        "not_found" => "not_found",
        "rate_limited" => "rate_limited",
        "validation_failed" => "validation_failed",
        "network_error" => "network_error",
        "server_error" => "server_error",
        "decode_error" => "decode_error",
        "confirmation_required" => "confirmation_required",
        "upstream_error" => "upstream_error",
        _ => "internal_error",
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_upstream_result, static_kind};
    use crate::dispatch::error::ToolError;
    use crate::mcp::envelope::build_error;
    use rmcp::ServerHandler;
    use rmcp::model::{CallToolResult, Content};

    /// Every kind that `ToolError::kind()` can return must have an explicit arm
    /// in `static_kind()`.  If a new variant or SDK kind is added to `ToolError`
    /// without a matching arm here, this test will catch the silent downgrade to
    /// `"internal_error"`.
    #[test]
    fn static_kind_round_trips_all_tool_error_kinds() {
        // Fixed-variant kinds — produced by the named ToolError variants.
        let fixed_variants: &[ToolError] = &[
            ToolError::UnknownAction {
                message: String::new(),
                valid: vec![],
                hint: None,
            },
            ToolError::MissingParam {
                message: String::new(),
                param: "p".into(),
            },
            ToolError::InvalidParam {
                message: String::new(),
                param: "p".into(),
            },
            ToolError::UnknownInstance {
                message: String::new(),
                valid: vec![],
            },
        ];

        for err in fixed_variants {
            let kind = err.kind();
            assert_eq!(
                static_kind(kind),
                kind,
                "static_kind({kind:?}) should round-trip but returns \"{}\"",
                static_kind(kind),
            );
        }

        // SDK-promoted kinds — every stable kind tag that `ApiError::kind()` can
        // return and that `ToolError::Sdk` promotes to the top-level `kind` field.
        let sdk_kinds: &[&str] = &[
            "unknown_action",
            "unknown_subaction",
            "missing_param",
            "invalid_param",
            "unknown_instance",
            "auth_failed",
            "not_found",
            "rate_limited",
            "validation_failed",
            "network_error",
            "server_error",
            "decode_error",
            "confirmation_required",
        ];

        for &sdk_kind in sdk_kinds {
            let err = ToolError::Sdk {
                sdk_kind: sdk_kind.to_string(),
                message: String::new(),
            };
            let kind = err.kind();
            assert_eq!(
                static_kind(kind),
                kind,
                "static_kind({kind:?}) should round-trip but returns \"{}\"",
                static_kind(kind),
            );
        }
    }

    #[test]
    fn normalize_upstream_result_preserves_user_errors_without_poisoning_health() {
        let upstream = CallToolResult::error(vec![Content::text(
            build_error("radarr", "movie.add", "missing_param", "need title").to_string(),
        )]);

        let (_, kind, counts_as_failure) =
            normalize_upstream_result("radarr", "call_tool", upstream);

        assert_eq!(kind, "missing_param");
        assert!(!counts_as_failure);
    }

    #[test]
    fn server_capabilities_advertise_list_changed_support() {
        let server = super::LabMcpServer {
            registry: std::sync::Arc::new(crate::registry::ToolRegistry::new()),
            gateway_manager: None,
            device_role: None,
            peers: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        };

        let info = server.get_info();
        assert_eq!(
            info.capabilities.tools.and_then(|c| c.list_changed),
            Some(true)
        );
        assert_eq!(
            info.capabilities.resources.and_then(|c| c.list_changed),
            Some(true)
        );
        assert_eq!(
            info.capabilities.prompts.and_then(|c| c.list_changed),
            Some(true)
        );
    }

    #[tokio::test]
    async fn server_reads_current_pool_from_gateway_manager() {
        let runtime = crate::dispatch::gateway::manager::GatewayRuntimeHandle::default();
        let manager = std::sync::Arc::new(crate::dispatch::gateway::manager::GatewayManager::new(
            std::path::PathBuf::from("config.toml"),
            runtime.clone(),
        ));
        let notifier = super::PeerNotifier::default();
        let server = super::LabMcpServer {
            registry: std::sync::Arc::new(crate::registry::ToolRegistry::new()),
            gateway_manager: Some(std::sync::Arc::clone(&manager)),
            device_role: None,
            peers: std::sync::Arc::clone(&notifier.peers),
        };

        assert!(server.current_upstream_pool().await.is_none());

        let pool = std::sync::Arc::new(crate::dispatch::upstream::pool::UpstreamPool::new());
        runtime.swap(Some(std::sync::Arc::clone(&pool))).await;

        let current = server.current_upstream_pool().await.expect("pool");
        assert!(std::sync::Arc::ptr_eq(&current, &pool));
    }

    #[tokio::test]
    async fn snapshot_catalog_hides_mcp_disabled_virtual_services() {
        let runtime = crate::dispatch::gateway::manager::GatewayRuntimeHandle::default();
        let manager = std::sync::Arc::new(crate::dispatch::gateway::manager::GatewayManager::new(
            std::path::PathBuf::from("config.toml"),
            runtime,
        ));
        manager
            .seed_config(crate::config::LabConfig {
                virtual_servers: vec![crate::config::VirtualServerConfig {
                    id: "plex".to_string(),
                    service: "plex".to_string(),
                    enabled: true,
                    surfaces: crate::config::VirtualServerSurfacesConfig {
                        cli: false,
                        api: false,
                        mcp: false,
                        webui: false,
                    },
                    mcp_policy: None,
                }],
                ..crate::config::LabConfig::default()
            })
            .await;

        let server = super::LabMcpServer {
            registry: std::sync::Arc::new(crate::registry::build_default_registry()),
            gateway_manager: Some(manager),
            device_role: None,
            peers: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        };

        let snapshot = server.snapshot_catalog().await;
        assert!(!snapshot.tools.contains("plex"));
    }

    #[tokio::test]
    async fn service_actions_json_filters_to_allowed_mcp_actions() {
        let runtime = crate::dispatch::gateway::manager::GatewayRuntimeHandle::default();
        let manager = std::sync::Arc::new(crate::dispatch::gateway::manager::GatewayManager::new(
            std::path::PathBuf::from("config.toml"),
            runtime,
        ));
        manager
            .seed_config(crate::config::LabConfig {
                virtual_servers: vec![crate::config::VirtualServerConfig {
                    id: "plex".to_string(),
                    service: "plex".to_string(),
                    enabled: true,
                    surfaces: crate::config::VirtualServerSurfacesConfig {
                        cli: false,
                        api: false,
                        mcp: true,
                        webui: false,
                    },
                    mcp_policy: Some(crate::config::VirtualServerMcpPolicyConfig {
                        allowed_actions: vec!["server.info".to_string()],
                    }),
                }],
                ..crate::config::LabConfig::default()
            })
            .await;

        let server = super::LabMcpServer {
            registry: std::sync::Arc::new(crate::registry::build_default_registry()),
            gateway_manager: Some(manager),
            device_role: None,
            peers: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        };

        let value = server
            .service_actions_json("plex")
            .await
            .expect("service actions");
        let actions = value.as_array().expect("array");
        assert!(actions.iter().any(|action| action["name"] == "help"));
        assert!(actions.iter().any(|action| action["name"] == "schema"));
        assert!(actions.iter().any(|action| action["name"] == "server.info"));
        assert!(
            !actions
                .iter()
                .any(|action| action["name"] == "session.list")
        );
    }
}
