//! `lab serve` — start the MCP server.

use std::process::ExitCode;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{Args, ValueEnum};
use rmcp::ServiceExt;

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

    match transport {
        Transport::Stdio => run_stdio(Arc::new(registry)).await,
        Transport::Http => {
            let bearer_token = http_token();
            let oauth_config = resolve_oauth(config.oauth.as_ref());
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

            // Wire OAuth JWT validation when configured.
            if let Some(ref oauth_cfg) = oauth_config {
                match crate::api::oauth::JwksManager::discover(
                    &oauth_cfg.issuer,
                    &oauth_cfg.audience,
                )
                .await
                {
                    Ok(jwks) => {
                        tracing::info!(
                            issuer = %oauth_cfg.issuer,
                            "OAuth JWKS discovery succeeded"
                        );
                        state = state.with_jwks(Arc::new(jwks));
                    }
                    Err(e) => {
                        tracing::error!(
                            issuer = %oauth_cfg.issuer,
                            error = %e,
                            "OAuth JWKS discovery failed — JWT validation will be unavailable"
                        );
                    }
                }
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
fn is_loopback_host(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "::1" | "localhost")
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
    let router = crate::api::router::build_router_with_bearer(state, bearer_token);
    // Parse and validate the address at bind time, not at CLI parse time.
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
    let clients = Arc::new(crate::api::state::ServiceClients::from_env());
    let server = LabMcpServer { registry, clients };
    let running = server.serve(rmcp::transport::stdio()).await?;
    running.waiting().await?;
    Ok(ExitCode::SUCCESS)
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
        assert!(is_loopback_host("localhost"));
        assert!(!is_loopback_host("0.0.0.0"));
        assert!(!is_loopback_host("192.168.1.100"));
        assert!(!is_loopback_host("lab.example.com"));
    }
}
