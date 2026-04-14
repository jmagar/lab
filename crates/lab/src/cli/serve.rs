//! `lab serve` — start the MCP server.

use std::process::ExitCode;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, OnceLock};

use anyhow::{Context, Result};
use clap::{Args, ValueEnum};
use rmcp::model::{
    AnnotateAble, CallToolRequestParams, CallToolResult, CompleteRequestParams, CompleteResult,
    CompletionInfo, Content, CreateElicitationRequestParams, ElicitationAction, ElicitationSchema,
    ErrorCode, GetPromptRequestParams, GetPromptResult, ListPromptsResult, ListResourcesResult,
    ListToolsResult, LoggingLevel, LoggingMessageNotificationParam, PaginatedRequestParams,
    PrimitiveSchema, RawResource, ReadResourceRequestParams, ReadResourceResult, Reference,
    ResourceContents, ServerCapabilities, ServerInfo, SetLevelRequestParams, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, Peer, RoleServer, ServerHandler, ServiceExt};
use serde_json::Value;

use crate::api::AppState;
use crate::config::LabConfig;
use crate::mcp::envelope::{build_error, build_error_extra, build_success};
use crate::mcp::error::DispatchError;
use crate::mcp::registry::{ToolRegistry, build_default_registry};

/// Sentinel severity value meaning "MCP logging disabled until the client opts in".
const LOG_DISABLED: u8 = u8::MAX;

/// Transport choices for `lab serve`.
#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum Transport {
    /// stdin/stdout framing (default, used by Claude Desktop etc.).
    Stdio,
    /// HTTP transport — requires `LAB_MCP_HTTP_TOKEN` in the environment.
    Http,
}

/// `lab serve` arguments.
#[derive(Debug, Args)]
pub struct ServeArgs {
    /// Comma- or space-separated list of services to enable. Empty = all.
    #[arg(long, value_delimiter = ',')]
    pub services: Vec<String>,
    /// Transport to use.
    #[arg(long, value_enum)]
    pub transport: Option<Transport>,
    /// Bind host for the HTTP transport.
    #[arg(long)]
    pub host: Option<String>,
    /// Bind port for the HTTP transport.
    #[arg(long)]
    pub port: Option<u16>,
}

/// Run the serve subcommand.
pub async fn run(args: ServeArgs, config: &LabConfig) -> Result<ExitCode> {
    let transport = resolve_transport(
        args.transport,
        std::env::var("LAB_MCP_TRANSPORT").ok(),
        config.mcp.transport.as_deref(),
    )?;
    // Resolve host and port here for source-of-truth ordering, but defer
    // address parsing and validation until the actual bind call in run_http.
    // This way an invalid host string only errors when HTTP transport is chosen,
    // not when --transport stdio is used (thread #2).
    let host = args
        .host
        .or_else(|| std::env::var("LAB_MCP_HTTP_HOST").ok())
        .or_else(|| config.mcp.host.clone())
        .unwrap_or_else(|| "127.0.0.1".to_string());
    if matches!(transport, Transport::Http) && host.is_empty() {
        anyhow::bail!("HTTP host cannot be empty — set LAB_MCP_HTTP_HOST or mcp.host in config");
    }
    let port = resolve_port(
        args.port,
        std::env::var("LAB_MCP_HTTP_PORT").ok(),
        config.mcp.port,
    )?;

    let registry = build_default_registry();
    let registry = filter_registry(registry, &args.services)?;

    match transport {
        Transport::Stdio => run_stdio(Arc::new(registry)).await,
        Transport::Http => {
            let state = AppState::from_registry(registry);
            run_http(
                &host,
                port,
                &require_http_token()?,
                state,
                &config.api.cors_origins,
            )
            .await
        }
    }
}

fn resolve_transport(
    cli: Option<Transport>,
    env: Option<String>,
    config: Option<&str>,
) -> Result<Transport> {
    if let Some(transport) = cli {
        return Ok(transport);
    }
    if let Some(value) = env {
        return Transport::from_str(&value, true)
            .map_err(|err| anyhow::anyhow!("invalid LAB_MCP_TRANSPORT value `{value}`: {err}"));
    }
    if let Some(value) = config {
        return Transport::from_str(value, true)
            .map_err(|err| anyhow::anyhow!("invalid mcp.transport value `{value}`: {err}"));
    }
    Ok(Transport::Stdio)
}

fn resolve_port(cli: Option<u16>, env: Option<String>, config: Option<u16>) -> Result<u16> {
    if let Some(port) = cli {
        return Ok(port);
    }
    if let Some(value) = env {
        return value
            .parse::<u16>()
            .with_context(|| format!("invalid LAB_MCP_HTTP_PORT value `{value}`"));
    }
    Ok(config.unwrap_or(8765))
}

fn require_http_token() -> Result<String> {
    std::env::var("LAB_MCP_HTTP_TOKEN")
        .ok()
        .filter(|value| !value.is_empty())
        .context("LAB_MCP_HTTP_TOKEN must be set when using --transport http")
}

fn filter_registry(registry: ToolRegistry, services: &[String]) -> Result<ToolRegistry> {
    if services.is_empty() {
        return Ok(registry);
    }
    let valid: Vec<&str> = registry.services().iter().map(|e| e.name).collect();
    let unknown: Vec<&str> = services
        .iter()
        .filter(|s| !valid.contains(&s.as_str()))
        .map(String::as_str)
        .collect();
    if !unknown.is_empty() {
        anyhow::bail!(
            "unknown service(s): {}. Valid services: {}",
            unknown.join(", "),
            valid.join(", ")
        );
    }
    let mut out = ToolRegistry::new();
    for entry in registry.services() {
        if services.iter().any(|s| s == entry.name) {
            out.register(entry.clone());
        }
    }
    Ok(out)
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
struct LabMcpServer {
    registry: Arc<ToolRegistry>,
    /// Captured on `on_initialized` — used to send logging notifications.
    peer: Arc<OnceLock<Peer<RoleServer>>>,
    /// Client-requested logging level as RFC 5424 severity. 255 = disabled.
    log_level: Arc<AtomicU8>,
}

impl ServerHandler for LabMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .enable_completions()
                .enable_logging()
                .build(),
        )
    }

    // ── Lifecycle ────────────────────────────────────────────────────

    async fn on_initialized(&self, context: rmcp::service::NotificationContext<RoleServer>) {
        let _previous_peer = self.peer.set(context.peer);
    }

    // ── Prompts ─────────────────────────────────────────────────────

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ListPromptsResult, ErrorData> {
        Ok(crate::mcp::prompts::list_all())
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<GetPromptResult, ErrorData> {
        let args = request
            .arguments
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| {
                let s = match v {
                    Value::String(s) => s,
                    other => other.to_string(),
                };
                (k, s)
            })
            .collect();
        crate::mcp::prompts::get(&self.registry, &request.name, &args).ok_or_else(|| {
            ErrorData::new(
                ErrorCode::INVALID_PARAMS,
                format!("unknown prompt: {}", request.name),
                None,
            )
        })
    }

    // ── Completions ─────────────────────────────────────────────────

    async fn complete(
        &self,
        request: CompleteRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<CompleteResult, ErrorData> {
        let argument_name = &request.argument.name;
        let partial = &request.argument.value;

        let values = match &request.r#ref {
            Reference::Prompt(prompt_ref) => {
                complete_prompt_arg(&prompt_ref.name, argument_name, partial, &self.registry)
            }
            Reference::Resource(_) => vec![],
        };
        let completion = CompletionInfo::with_all_values(values).unwrap_or_default();
        Ok(CompleteResult::new(completion))
    }

    // ── Logging ─────────────────────────────────────────────────────

    async fn set_level(
        &self,
        request: SetLevelRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<(), ErrorData> {
        self.log_level
            .store(level_to_severity(request.level), Ordering::Relaxed);
        tracing::debug!(level = ?request.level, "MCP client set logging level");
        Ok(())
    }

    // ── Resources ───────────────────────────────────────────────────

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
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
        Ok(ListResourcesResult::with_all_items(resources))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        let uri = &request.uri;
        let json = if uri == "lab://catalog" {
            crate::mcp::resources::catalog_json(&self.registry)
        } else if let Some(service) = uri
            .strip_prefix("lab://")
            .and_then(|s| s.strip_suffix("/actions"))
        {
            crate::mcp::resources::service_actions_json(&self.registry, service)
        } else {
            return Err(ErrorData::new(
                ErrorCode::RESOURCE_NOT_FOUND,
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
            Err(e) => Err(ErrorData::new(
                ErrorCode::INTERNAL_ERROR,
                e.to_string(),
                None,
            )),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let schema = Arc::new(action_schema());
        let tools: Vec<Tool> = self
            .registry
            .services()
            .iter()
            .map(|svc| Tool::new(svc.name, svc.description, Arc::clone(&schema)))
            .collect();
        Ok(ListToolsResult::with_all_items(tools))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let service = request.name.as_ref().to_string();
        let args = request.arguments.unwrap_or_default();
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let params = args.get("params").cloned().unwrap_or(Value::Null);

        let svc = self.registry.services().iter().find(|s| s.name == service);

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
                        // Automated agents (CI, Claude Desktop non-interactive, headless MCP)
                        // can opt in by passing confirm:true in the action params.
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
                }
            }
        }

        let start = std::time::Instant::now();
        let result = match svc {
            Some(entry) => (entry.dispatch)(action.clone(), params)
                .await
                .map_err(|te| anyhow::Error::from(DispatchError::from(te))),
            None => Err(anyhow::anyhow!(
                "service `{service}` has no dispatcher wired"
            )),
        };
        let elapsed_ms = start.elapsed().as_millis();

        let (call_result, log_level, log_err) =
            dispatch_to_result(&service, &action, elapsed_ms, result);

        self.maybe_notify_log(log_level, &service, &action, elapsed_ms, log_err.as_deref())
            .await;

        call_result
    }
}

impl LabMcpServer {
    /// Send an MCP logging notification if the client threshold permits it.
    ///
    /// Takes raw components instead of a pre-built `Value` to avoid heap
    /// allocation on every dispatch when logging is disabled (the common case).
    async fn maybe_notify_log(
        &self,
        level: LoggingLevel,
        service: &str,
        action: &str,
        elapsed_ms: u128,
        error: Option<&str>,
    ) {
        let client_severity = self.log_level.load(Ordering::Relaxed);
        if client_severity != LOG_DISABLED
            && level_to_severity(level) <= client_severity
            && let Some(peer) = self.peer.get()
        {
            let data = error.map_or_else(
                || {
                    serde_json::json!({
                        "service": service, "action": action, "elapsed_ms": elapsed_ms,
                    })
                },
                |err| {
                    serde_json::json!({
                        "service": service, "action": action,
                        "elapsed_ms": elapsed_ms, "error": err,
                    })
                },
            );
            let _notify_result = peer
                .notify_logging_message(LoggingMessageNotificationParam {
                    level,
                    data,
                    logger: Some("lab".to_string()),
                })
                .await;
        }
    }
}

/// Convert a dispatch result into a `CallToolResult` + logging metadata.
///
/// Returns the error message (if any) separately so `maybe_notify_log` can
/// build the JSON `Value` only when a notification will actually be sent.
fn dispatch_to_result(
    service: &str,
    action: &str,
    elapsed_ms: u128,
    result: Result<Value>,
) -> (
    std::result::Result<CallToolResult, ErrorData>,
    LoggingLevel,
    Option<String>,
) {
    match result {
        Ok(v) => {
            tracing::info!(surface = "mcp", service, action, elapsed_ms, "dispatch ok");
            let envelope = build_success(service, action, &v);
            let call = Ok(CallToolResult::success(vec![Content::text(
                envelope.to_string(),
            )]));
            (call, LoggingLevel::Info, None)
        }
        Err(e) => {
            let (kind, message, extra) = extract_error_info(&e);
            let is_fatal = matches!(kind, "internal_error" | "server_error" | "decode_error");
            let level = if is_fatal {
                LoggingLevel::Error
            } else {
                LoggingLevel::Warning
            };
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
            let call = Ok(CallToolResult::error(vec![Content::text(
                envelope.to_string(),
            )]));
            (
                call,
                level,
                Some(format!("{kind}: {}", sanitize_log_error(kind, &message))),
            )
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
        Err(_) => ElicitResult::NotSupported,
    }
}

/// Map `LoggingLevel` to RFC 5424 severity (Emergency=0, Debug=7).
///
/// **Do not use `as u8`** — rmcp's variant ordering is Debug=0..Emergency=7,
/// which is the inverse of RFC 5424. An `as u8` cast would invert comparisons.
const fn level_to_severity(level: LoggingLevel) -> u8 {
    match level {
        LoggingLevel::Emergency => 0,
        LoggingLevel::Alert => 1,
        LoggingLevel::Critical => 2,
        LoggingLevel::Error => 3,
        LoggingLevel::Warning => 4,
        LoggingLevel::Notice => 5,
        LoggingLevel::Info => 6,
        LoggingLevel::Debug => 7,
    }
}

/// Complete a prompt argument value.
fn complete_prompt_arg(
    prompt_name: &str,
    argument_name: &str,
    partial: &str,
    registry: &ToolRegistry,
) -> Vec<String> {
    match (prompt_name, argument_name) {
        // service arg on any prompt → service names
        (_, "service") => registry
            .services()
            .iter()
            .map(|s| s.name.to_string())
            .filter(|n| n.starts_with(partial))
            .collect(),
        // action arg on run-action → action names (need service context from prior args,
        // but MCP completions don't provide that yet — return all action names across services)
        ("run-action", "action") => registry
            .sorted_action_names()
            .iter()
            .filter(|name| name.starts_with(partial))
            .cloned()
            .collect(),
        _ => vec![],
    }
}

fn sanitize_log_error(kind: &str, message: &str) -> String {
    let single_line = message.lines().next().unwrap_or_default().trim();
    let redacted = redact_home_paths(single_line);
    let sanitized = if matches!(kind, "internal_error" | "server_error") {
        "internal server error".to_string()
    } else if redacted.is_empty() {
        kind.replace('_', " ")
    } else {
        redacted
    };

    const MAX_LOG_ERROR_LEN: usize = 200;
    if sanitized.chars().count() > MAX_LOG_ERROR_LEN {
        let truncated: String = sanitized.chars().take(MAX_LOG_ERROR_LEN - 1).collect();
        format!("{truncated}…")
    } else {
        sanitized
    }
}

fn redact_home_paths(message: &str) -> String {
    let mut out = message.to_string();
    if let Some(home) = std::env::var_os("HOME").and_then(|value| value.into_string().ok())
        && !home.is_empty()
    {
        out = out.replace(&home, "~");
    }
    out
}

async fn run_http(
    host: &str,
    port: u16,
    bearer_token: &str,
    state: AppState,
    cors_origins: &[String],
) -> Result<ExitCode> {
    let router =
        crate::api::router::build_router(state, Some(bearer_token.to_string()), cors_origins);
    // Parse and validate the address at bind time, not at CLI parse time.
    // This defers host/port errors to when the HTTP transport is actually used
    // and gives tokio a chance to surface OS-level bind errors directly.
    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind HTTP listener on `{addr}`"))?;
    tracing::info!(addr, "lab serve (http) ready");
    axum::serve(listener, router).await?;
    Ok(ExitCode::SUCCESS)
}

async fn run_stdio(registry: Arc<ToolRegistry>) -> Result<ExitCode> {
    tracing::info!(
        services = registry.services().len(),
        "lab serve (stdio) ready"
    );
    let server = LabMcpServer {
        registry,
        peer: Arc::new(OnceLock::new()),
        log_level: Arc::new(AtomicU8::new(LOG_DISABLED)),
    };
    let running = server.serve(rmcp::transport::stdio()).await?;
    running.waiting().await?;
    Ok(ExitCode::SUCCESS)
}

/// Recover a stable kind tag and message from an `anyhow::Error`.
///
/// Priority:
/// 1. Downcast to [`DispatchError`] — gives structured kind + optional extras.
/// 2. Parse `e.to_string()` as JSON `{ "kind": "…" }` — covers `ToolError`
///    errors that were serialized to string before entering anyhow (radarr).
/// 3. Fall back to `"internal_error"`.
fn extract_error_info(e: &anyhow::Error) -> (&'static str, String, Option<Value>) {
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
fn static_kind(s: &str) -> &'static str {
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
        _ => "internal_error",
    }
}

#[cfg(test)]
mod tests {
    use super::static_kind;
    use super::{
        LOG_DISABLED, Transport, complete_prompt_arg, extract_error_info, redact_home_paths,
        resolve_port, resolve_transport, sanitize_log_error,
    };
    use crate::config::{LabConfig, McpPreferences};
    use crate::dispatch::error::ToolError;
    use crate::mcp::error::DispatchError;
    use crate::mcp::registry::{RegisteredService, ToolRegistry};
    use lab_apis::core::action::ActionSpec;
    use serde_json::Value;

    const ACTIONS_ALPHA: &[ActionSpec] = &[
        ActionSpec {
            name: "alpha.read",
            description: "Read alpha data",
            destructive: false,
            params: &[],
            returns: "object",
        },
        ActionSpec {
            name: "alpha.write",
            description: "Write alpha data",
            destructive: true,
            params: &[],
            returns: "object",
        },
    ];

    const ACTIONS_BETA: &[ActionSpec] = &[ActionSpec {
        name: "beta.list",
        description: "List beta data",
        destructive: false,
        params: &[],
        returns: "object[]",
    }];

    fn registry() -> ToolRegistry {
        let mut registry = ToolRegistry::new();
        registry.register(RegisteredService {
            name: "alpha",
            description: "alpha service",
            category: "test",
            status: "available",
            actions: ACTIONS_ALPHA,
            dispatch: |_action, _params| Box::pin(async { Ok(Value::Null) }),
        });
        registry.register(RegisteredService {
            name: "beta",
            description: "beta service",
            category: "test",
            status: "available",
            actions: ACTIONS_BETA,
            dispatch: |_action, _params| Box::pin(async { Ok(Value::Null) }),
        });
        registry
    }

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
        // This list must stay in sync with the arms in `static_kind()`.
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
    fn transport_resolution_prefers_cli_then_env_then_config() {
        let resolved =
            resolve_transport(Some(Transport::Http), Some("stdio".into()), Some("stdio"))
                .expect("cli value should win");
        assert!(matches!(resolved, Transport::Http));

        let resolved = resolve_transport(None, Some("http".into()), Some("stdio"))
            .expect("env value should win");
        assert!(matches!(resolved, Transport::Http));

        let resolved =
            resolve_transport(None, None, Some("http")).expect("config value should win");
        assert!(matches!(resolved, Transport::Http));
    }

    #[test]
    fn port_resolution_prefers_cli_then_env_then_config() {
        assert_eq!(
            resolve_port(Some(9999), Some("8888".into()), Some(7777)).unwrap(),
            9999
        );
        assert_eq!(
            resolve_port(None, Some("8888".into()), Some(7777)).unwrap(),
            8888
        );
        assert_eq!(resolve_port(None, None, Some(7777)).unwrap(), 7777);
        assert_eq!(resolve_port(None, None, None).unwrap(), 8765);
    }

    #[test]
    fn config_defaults_are_available_for_serve_resolution() {
        let cfg = LabConfig {
            mcp: McpPreferences {
                transport: Some("http".into()),
                host: Some("0.0.0.0".into()),
                port: Some(9000),
            },
            ..LabConfig::default()
        };
        assert_eq!(cfg.mcp.host.as_deref(), Some("0.0.0.0"));
    }

    #[test]
    fn completion_handler_reuses_cached_sorted_actions() {
        let registry = registry();

        let all = complete_prompt_arg("run-action", "action", "", &registry);
        assert_eq!(
            all,
            vec![
                "alpha.read".to_string(),
                "alpha.write".to_string(),
                "beta.list".to_string()
            ]
        );

        let filtered = complete_prompt_arg("run-action", "action", "alpha.", &registry);
        assert_eq!(
            filtered,
            vec!["alpha.read".to_string(), "alpha.write".to_string()]
        );
    }

    #[test]
    fn sanitize_log_error_redacts_and_flattens_messages() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/tester".to_string());
        let raw = format!("boom at {home}/workspace/lab\nstack trace line 2");

        let sanitized = sanitize_log_error("network_error", &raw);

        assert!(!sanitized.contains(&home));
        assert!(sanitized.contains("~/workspace/lab"));
        assert!(!sanitized.contains("stack trace line 2"));
    }

    #[test]
    fn sanitize_log_error_hides_internal_details() {
        let sanitized = sanitize_log_error(
            "internal_error",
            "decode panic at /home/jmagar/workspace/lab/crates/lab/src/cli/serve.rs:123",
        );
        assert_eq!(sanitized, "internal server error");
    }

    #[test]
    fn redact_home_paths_replaces_user_home() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/tester".to_string());
        let redacted = redact_home_paths(&format!("see {home}/secret/file"));
        assert!(!redacted.contains(&home));
        assert!(redacted.contains("~/secret/file"));
    }

    #[tokio::test]
    async fn extract_error_info_preserves_dispatch_error_downcast() {
        let tool_error = crate::mcp::services::extract::dispatch("missing.action", Value::Null)
            .await
            .expect_err("unknown action should fail");
        let err = anyhow::Error::from(DispatchError::from(tool_error));

        let (kind, message, extra) = extract_error_info(&err);

        assert_eq!(kind, "unknown_action");
        assert!(message.contains("missing.action"));
        let extra = extra.expect("unknown action should carry valid actions");
        assert!(extra["valid"].is_array());
    }

    #[test]
    fn extract_error_info_preserves_json_fallback() {
        let err = anyhow::anyhow!(
            "{}",
            ToolError::MissingParam {
                message: "missing required parameter `query`".to_string(),
                param: "query".to_string(),
            }
        );

        let (kind, message, extra) = extract_error_info(&err);

        assert_eq!(kind, "missing_param");
        assert_eq!(message, "missing required parameter `query`");
        assert_eq!(extra.expect("param metadata")["param"], "query");
    }

    #[test]
    fn log_disabled_constant_uses_max_sentinel() {
        assert_eq!(LOG_DISABLED, u8::MAX);
    }
}
