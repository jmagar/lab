//! `lab serve` — start the MCP server.

use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Args, ValueEnum};
use rmcp::ServiceExt;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService,
    session::local::{LocalSessionManager, SessionConfig},
};

use crate::api::AppState;
use crate::config::{LabConfig, resolve_oauth};
use crate::mcp::registry::{ToolRegistry, build_default_registry};
use crate::mcp::server::LabMcpServer;

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

    // Build upstream pool from config if any [[upstream]] entries exist.
    let upstream_pool = if config.upstream.is_empty() {
        None
    } else {
        let pool =
            crate::dispatch::upstream::pool::UpstreamPool::new();
        pool.discover_all(&config.upstream).await;
        Some(Arc::new(pool))
    };

    match transport {
        Transport::Stdio => run_stdio(Arc::new(registry), upstream_pool).await,
        Transport::Http => {
            let bearer_token = http_token();
            let oauth_config = resolve_oauth(config.oauth.as_ref())
                .context("invalid OAuth configuration")?;
            let auth_configured = bearer_token.is_some() || oauth_config.is_some();

            // Safety gate: refuse to bind on a non-localhost address without
            // any auth configured (lab-319g). This prevents accidental
            // unauthenticated deployment on a LAN-accessible address.
            if !auth_configured && !is_loopback_host(&host) {
                anyhow::bail!(
                    "refusing to bind HTTP on {host}:{port} without authentication. \
                     Set LAB_MCP_HTTP_TOKEN or LAB_OAUTH_ISSUER, or bind to \
                     127.0.0.1 for local-only access."
                );
            }

            let mut state = AppState::from_registry(registry);

            // Wire upstream pool into state for both HTTP and MCP surfaces.
            if let Some(ref pool) = upstream_pool {
                state = state.with_upstream_pool(Arc::clone(pool));
            }

            // Wire OAuth JWT validation when configured.
            if let Some(oauth_cfg) = oauth_config {
                let jwks = crate::api::oauth::JwksManager::discover(
                    &oauth_cfg.issuer,
                    &oauth_cfg.audience,
                )
                .await
                .with_context(|| {
                    format!(
                        "OAuth JWKS discovery failed for issuer `{}` — \
                         refusing to start without JWT validation",
                        oauth_cfg.issuer
                    )
                })?;
                tracing::info!(
                    issuer = %oauth_cfg.issuer,
                    "OAuth JWKS discovery succeeded"
                );
                state = state.with_jwks(Arc::new(jwks));
                state = state.with_oauth_config(oauth_cfg);
            }

            run_http(&host, port, bearer_token, state).await
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
    let normalized = host.trim_start_matches('[').trim_end_matches(']');
    matches!(normalized, "127.0.0.1" | "::1" | "localhost")
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
) -> Result<ExitCode> {
    // Build the MCP streamable HTTP service in the serve path (not in the
    // router module) to avoid an api->mcp dependency.
    let mcp_service = build_mcp_service(&state);
    let mcp_router = axum::Router::new().nest_service("/mcp", mcp_service);
    let router =
        crate::api::router::build_router_with_bearer(state, bearer_token, Some(mcp_router));
    // Parse and validate the address at bind time, not at CLI parse time.
    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind HTTP listener on `{addr}`"))?;
    tracing::info!(addr, "lab serve (http) ready");
    axum::serve(listener, router).await?;
    Ok(ExitCode::SUCCESS)
}

async fn run_stdio(
    registry: Arc<ToolRegistry>,
    upstream_pool: Option<Arc<crate::dispatch::upstream::pool::UpstreamPool>>,
) -> Result<ExitCode> {
    tracing::info!(
        services = registry.services().len(),
        "lab serve (stdio) ready"
    );
    let clients = Arc::new(crate::api::state::ServiceClients::from_env());
    let server = LabMcpServer {
        registry,
        clients,
        upstream_pool,
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
) -> StreamableHttpService<LabMcpServer, LocalSessionManager> {
    let registry = Arc::clone(&state.registry);
    let clients = Arc::clone(&state.clients);
    let upstream_pool = state.upstream_pool.clone();

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
        .with_allowed_hosts(allowed_hosts_from_env())
        .with_stateful_mode(stateful);

    StreamableHttpService::new(
        move || {
            let reg = Arc::clone(&registry);
            let cl = Arc::clone(&clients);
            let pool = upstream_pool.clone();
            Ok(LabMcpServer {
                registry: reg,
                clients: cl,
                upstream_pool: pool,
            })
        },
        session_manager,
        config,
    )
}

/// Build the allowed hosts list for DNS rebinding protection.
///
/// Reads `LAB_MCP_ALLOWED_HOSTS` (comma-separated) and `LAB_RESOURCE_URL`
/// from the environment. Always includes loopback defaults. Rejects wildcard.
fn allowed_hosts_from_env() -> Vec<String> {
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
    // If LAB_RESOURCE_URL is set, auto-extract and add its hostname
    if let Ok(url_str) = std::env::var("LAB_RESOURCE_URL")
        && let Ok(parsed) = url::Url::parse(&url_str)
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
    use super::{Transport, is_loopback_host, resolve_port, resolve_transport};
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
}
