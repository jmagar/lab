//! `lab serve` — start the MCP server.

use std::future::Future;
use std::process::ExitCode;
use std::sync::Arc;

use anyhow::Result;
use clap::{Args, ValueEnum};
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, ListToolsResult, PaginatedRequestParams,
    ServerInfo, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler, ServiceExt};
use serde_json::Value;

use crate::mcp::registry::{ToolRegistry, build_default_registry};

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
    #[arg(long, value_enum, default_value_t = Transport::Stdio)]
    pub transport: Transport,
    /// Bind host for the HTTP transport.
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
    /// Bind port for the HTTP transport.
    #[arg(long, default_value_t = 8765)]
    pub port: u16,
}

/// Run the serve subcommand.
pub async fn run(args: ServeArgs) -> Result<ExitCode> {
    let registry = build_default_registry();
    let registry = filter_registry(registry, &args.services);
    let registry = Arc::new(registry);

    match args.transport {
        Transport::Stdio => run_stdio(registry).await,
        Transport::Http => {
            tracing::warn!(host = %args.host, port = args.port, "http transport not yet wired");
            Ok(ExitCode::from(64))
        }
    }
}

fn filter_registry(registry: ToolRegistry, services: &[String]) -> ToolRegistry {
    if services.is_empty() {
        return registry;
    }
    let mut out = ToolRegistry::new();
    for entry in registry.services() {
        if services.iter().any(|s| s == entry.name) {
            out.register(entry.clone());
        }
    }
    out
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
}

impl ServerHandler for LabMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::default()
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_ {
        let schema = Arc::new(action_schema());
        let tools: Vec<Tool> = self
            .registry
            .services()
            .iter()
            .map(|svc| Tool::new(svc.name, svc.description, Arc::clone(&schema)))
            .collect();
        async move { Ok(ListToolsResult::with_all_items(tools)) }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResult, ErrorData>> + Send + '_ {
        let registry = Arc::clone(&self.registry);
        async move {
            let service = request.name.as_ref().to_string();
            let args = request.arguments.unwrap_or_default();
            let action = args
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let params = args.get("params").cloned().unwrap_or(Value::Null);

            let start = std::time::Instant::now();
            let result = dispatch_service(&registry, &service, &action, params).await;
            let elapsed_ms = start.elapsed().as_millis();

            match result {
                Ok(v) => {
                    tracing::info!(service, action, elapsed_ms, "dispatch ok");
                    Ok(CallToolResult::success(vec![Content::text(v.to_string())]))
                }
                Err(e) => {
                    // Error message is serialized ToolError JSON — extract kind for the log field.
                    let kind = serde_json::from_str::<Value>(&e.to_string())
                        .ok()
                        .and_then(|v| v["kind"].as_str().map(ToOwned::to_owned))
                        .unwrap_or_else(|| "internal_error".to_string());
                    tracing::warn!(service, action, elapsed_ms, kind, "dispatch error");
                    Ok(CallToolResult::error(vec![Content::text(e.to_string())]))
                }
            }
        }
    }
}

async fn run_stdio(registry: Arc<ToolRegistry>) -> Result<ExitCode> {
    tracing::info!(
        services = registry.services().len(),
        "lab serve (stdio) ready"
    );
    let server = LabMcpServer { registry };
    let running = server.serve(rmcp::transport::stdio()).await?;
    running.waiting().await?;
    Ok(ExitCode::SUCCESS)
}

async fn dispatch_service(
    registry: &ToolRegistry,
    service: &str,
    action: &str,
    params: Value,
) -> Result<Value> {
    if !registry.services().iter().any(|s| s.name == service) {
        anyhow::bail!("unknown service `{service}`");
    }
    match service {
        "extract" => crate::mcp::services::extract::dispatch(action, params).await,
        #[cfg(feature = "radarr")]
        "radarr" => crate::mcp::services::radarr::dispatch(action, params)
            .await
            .map_err(|te| {
                anyhow::anyhow!(
                    "{}",
                    serde_json::to_string(&te).unwrap_or_else(|_| format!("{te:?}"))
                )
            }),
        #[cfg(feature = "sonarr")]
        "sonarr" => crate::mcp::services::sonarr::dispatch(action, params).await,
        #[cfg(feature = "prowlarr")]
        "prowlarr" => crate::mcp::services::prowlarr::dispatch(action, params).await,
        #[cfg(feature = "plex")]
        "plex" => crate::mcp::services::plex::dispatch(action, params).await,
        #[cfg(feature = "tautulli")]
        "tautulli" => crate::mcp::services::tautulli::dispatch(action, params).await,
        #[cfg(feature = "sabnzbd")]
        "sabnzbd" => crate::mcp::services::sabnzbd::dispatch(action, params).await,
        #[cfg(feature = "qbittorrent")]
        "qbittorrent" => crate::mcp::services::qbittorrent::dispatch(action, params).await,
        #[cfg(feature = "tailscale")]
        "tailscale" => crate::mcp::services::tailscale::dispatch(action, params).await,
        #[cfg(feature = "linkding")]
        "linkding" => crate::mcp::services::linkding::dispatch(action, params).await,
        #[cfg(feature = "memos")]
        "memos" => crate::mcp::services::memos::dispatch(action, params).await,
        #[cfg(feature = "bytestash")]
        "bytestash" => crate::mcp::services::bytestash::dispatch(action, params).await,
        #[cfg(feature = "paperless")]
        "paperless" => crate::mcp::services::paperless::dispatch(action, params).await,
        #[cfg(feature = "arcane")]
        "arcane" => crate::mcp::services::arcane::dispatch(action, params).await,
        #[cfg(feature = "unraid")]
        "unraid" => crate::mcp::services::unraid::dispatch(action, params).await,
        #[cfg(feature = "unifi")]
        "unifi" => crate::mcp::services::unifi::dispatch(action, params).await,
        #[cfg(feature = "overseerr")]
        "overseerr" => crate::mcp::services::overseerr::dispatch(action, params).await,
        #[cfg(feature = "gotify")]
        "gotify" => crate::mcp::services::gotify::dispatch(action, params).await,
        #[cfg(feature = "openai")]
        "openai" => crate::mcp::services::openai::dispatch(action, params).await,
        #[cfg(feature = "qdrant")]
        "qdrant" => crate::mcp::services::qdrant::dispatch(action, params).await,
        #[cfg(feature = "tei")]
        "tei" => crate::mcp::services::tei::dispatch(action, params).await,
        #[cfg(feature = "apprise")]
        "apprise" => crate::mcp::services::apprise::dispatch(action, params).await,
        other => anyhow::bail!("service `{other}` has no dispatcher wired"),
    }
}
