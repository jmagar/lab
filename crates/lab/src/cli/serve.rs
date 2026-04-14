//! `lab serve` — start the MCP server.

use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Args, ValueEnum};
use lab_auth::config::AuthMode;
use rmcp::ServiceExt;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService,
    session::local::{LocalSessionManager, SessionConfig},
};
use tokio::sync::mpsc;

use crate::api::AppState;
use crate::config::{LabConfig, config_toml_path, resolve_auth};
use crate::dispatch::gateway::install_gateway_manager;
use crate::dispatch::gateway::manager::{GatewayManager, GatewayRuntimeHandle};
use crate::dispatch::gateway::types::CatalogChangeNotifier;
use crate::mcp::server::{LabMcpServer, PeerNotifier};
use crate::registry::{ToolRegistry, build_default_registry};

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

    let gateway_runtime = GatewayRuntimeHandle::default();
    if !config.upstream.is_empty() {
        let pool = Arc::new(crate::dispatch::upstream::pool::UpstreamPool::new());
        pool.discover_all(&config.upstream).await;
        gateway_runtime.swap(Some(pool)).await;
    }
    let notifier = PeerNotifier::default();
    let (notify_tx, notify_rx) = mpsc::unbounded_channel();
    let _catalog_notifier_task = tokio::spawn(notifier.clone().run(notify_rx));
    let mut gateway_manager = GatewayManager::new(
        config_toml_path().unwrap_or_else(|| "config.toml".into()),
        gateway_runtime.clone(),
    );
    gateway_manager.set_notifier(CatalogChangeNotifier::new(notify_tx));
    let gateway_manager = Arc::new(gateway_manager);
    gateway_manager.seed_config(config.clone()).await;
    install_gateway_manager(Arc::clone(&gateway_manager));

    match transport {
        Transport::Stdio => {
            run_stdio(Arc::new(registry), Arc::clone(&gateway_manager), notifier).await
        }
        Transport::Http => {
            let bearer_token = http_token();
            let auth_config =
                resolve_auth(config.auth.as_ref()).context("invalid HTTP auth configuration")?;
            let auth_configured =
                bearer_token.is_some() || matches!(auth_config.mode, AuthMode::OAuth);

            // Safety gate: refuse to bind on a non-localhost address without
            // any auth configured (lab-319g). This prevents accidental
            // unauthenticated deployment on a LAN-accessible address.
            if !auth_configured && !is_loopback_host(&host) {
                anyhow::bail!(
                    "refusing to bind HTTP on {host}:{port} without authentication. \
                     Set LAB_MCP_HTTP_TOKEN or LAB_AUTH_MODE=oauth, or bind to \
                     127.0.0.1 for local-only access."
                );
            }

            let oauth_state = if matches!(auth_config.mode, AuthMode::OAuth) {
                Some(
                    lab_auth::state::AuthState::new(auth_config.clone())
                        .await
                        .context("initialize lab-auth oauth state")?,
                )
            } else {
                None
            };

            let mut state = AppState::from_registry(registry);
            state = state.with_gateway_manager(Arc::clone(&gateway_manager));
            state = state.with_auth_config(auth_config);

            run_http(
                &host,
                port,
                bearer_token,
                state,
                oauth_state,
                &config.api.cors_origins,
                notifier,
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

/// Return the bearer token if configured, or `None` for auth-free operation.
fn http_token() -> Option<String> {
    std::env::var("LAB_MCP_HTTP_TOKEN")
        .ok()
        .filter(|value| !value.is_empty())
}

/// Check whether a host string refers to a loopback address.
///
/// Handles both bare and bracketed IPv6 (e.g. `::1` and `[::1]`).
fn is_loopback_host(host: &str) -> bool {
    let normalized = host.trim().trim_start_matches('[').trim_end_matches(']');
    matches!(normalized, "127.0.0.1" | "::1" | "localhost")
}

fn bind_addr(host: &str, port: u16) -> String {
    if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
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

async fn run_http(
    host: &str,
    port: u16,
    bearer_token: Option<String>,
    state: AppState,
    auth_state: Option<lab_auth::state::AuthState>,
    config_cors_origins: &[String],
    notifier: PeerNotifier,
) -> Result<ExitCode> {
    // Build the MCP streamable HTTP service in the serve path (not in the
    // router module) to avoid an api->mcp dependency.
    let mcp_service = build_mcp_service(&state, notifier);
    let mcp_router = axum::Router::new().nest_service("/mcp", mcp_service);
    let router = crate::api::router::build_router(
        state,
        bearer_token,
        auth_state,
        Some(mcp_router),
        config_cors_origins,
    );
    // Parse and validate the address at bind time, not at CLI parse time.
    let addr = bind_addr(host, port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind HTTP listener on `{addr}`"))?;
    tracing::info!(addr, "lab serve (http) ready");
    axum::serve(listener, router).await?;
    Ok(ExitCode::SUCCESS)
}

async fn run_stdio(
    registry: Arc<ToolRegistry>,
    gateway_manager: Arc<GatewayManager>,
    notifier: PeerNotifier,
) -> Result<ExitCode> {
    tracing::info!(
        services = registry.services().len(),
        "lab serve (stdio) ready"
    );
    let server = LabMcpServer {
        registry,
        gateway_manager: Some(Arc::clone(&gateway_manager)),
        peers: Arc::clone(&notifier.peers),
    };
    let running = server.serve(rmcp::transport::stdio()).await?;
    running.waiting().await?;
    Ok(ExitCode::SUCCESS)
}

/// Build the MCP streamable HTTP service from app state.
///
/// The factory closure clones `Arc<ToolRegistry>` from `AppState` and constructs
/// a new `LabMcpServer` per session. Construction cost: two Arc increments.
fn build_mcp_service(
    state: &AppState,
    notifier: PeerNotifier,
) -> StreamableHttpService<LabMcpServer, LocalSessionManager> {
    let registry = Arc::clone(&state.registry);
    let gateway_manager = state.gateway_manager.clone();

    let session_ttl_secs: u64 = std::env::var("LAB_MCP_SESSION_TTL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300);

    let mut session_config = SessionConfig::default();
    session_config.keep_alive = Some(Duration::from_secs(session_ttl_secs));

    let mut session_manager = LocalSessionManager::default();
    session_manager.session_config = session_config;
    let session_manager = Arc::new(session_manager);

    let stateful = std::env::var("LAB_MCP_STATEFUL")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(true);

    let config = StreamableHttpServerConfig::default()
        .with_allowed_hosts(allowed_hosts(
            state
                .auth_config
                .as_ref()
                .and_then(|cfg| cfg.public_url.as_ref().map(url::Url::as_str)),
        ))
        .with_stateful_mode(stateful);

    // All HTTP sessions share the same PeerNotifier (and thus the same peers
    // vec) so that gateway reload notifications reach every connected session.
    let shared_peers = Arc::clone(&notifier.peers);

    StreamableHttpService::new(
        move || {
            let reg = Arc::clone(&registry);
            let manager = gateway_manager.clone();
            let peers = Arc::clone(&shared_peers);
            Ok(LabMcpServer {
                registry: reg,
                gateway_manager: manager,
                peers,
            })
        },
        session_manager,
        config,
    )
}

/// Build the allowed hosts list for DNS rebinding protection.
///
/// Reads `LAB_MCP_ALLOWED_HOSTS` (comma-separated) and the resolved resource
/// URL. Always includes loopback defaults. Rejects wildcard.
fn allowed_hosts(resource_url: Option<&str>) -> Vec<String> {
    let mut hosts = vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "::1".to_string(),
    ];
    if let Ok(extra) = std::env::var("LAB_MCP_ALLOWED_HOSTS") {
        for h in extra.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            // Reject wildcard — would disable Host header validation entirely
            if h == "*" {
                tracing::warn!(
                    "ignoring wildcard '*' in LAB_MCP_ALLOWED_HOSTS — \
                     would disable DNS rebinding protection"
                );
                continue;
            }
            if !hosts.contains(&h.to_string()) {
                hosts.push(h.to_string());
            }
        }
    }
    if let Some(url_str) = resource_url
        && let Ok(parsed) = url::Url::parse(url_str)
        && let Some(host) = parsed.host_str()
    {
        let h = host.to_string();
        if !hosts.contains(&h) {
            hosts.push(h);
        }
    }
    hosts
}

#[cfg(test)]
mod tests {
    use super::{
        Transport, allowed_hosts, bind_addr, is_loopback_host, resolve_port, resolve_transport,
    };
    use crate::config::{LabConfig, McpPreferences};

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
    fn loopback_host_detection() {
        assert!(is_loopback_host("127.0.0.1"));
        assert!(is_loopback_host("::1"));
        assert!(is_loopback_host("[::1]"));
        assert!(is_loopback_host("localhost"));
        assert!(!is_loopback_host("0.0.0.0"));
        assert!(!is_loopback_host("192.168.1.100"));
        assert!(!is_loopback_host("lab.example.com"));
    }

    #[test]
    fn bind_addr_brackets_bare_ipv6_hosts() {
        assert_eq!(bind_addr("::1", 8765), "[::1]:8765");
        assert_eq!(bind_addr("[::1]", 8765), "[::1]:8765");
        assert_eq!(bind_addr("127.0.0.1", 8765), "127.0.0.1:8765");
    }

    #[test]
    fn allowed_hosts_include_resource_url_host() {
        let hosts = allowed_hosts(Some("https://lab.example.com/mcp"));
        assert!(hosts.contains(&"lab.example.com".to_string()));
    }
}
