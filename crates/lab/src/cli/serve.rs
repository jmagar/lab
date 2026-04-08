//! `lab serve` — start the MCP server.

use std::process::ExitCode;

use anyhow::Result;
use clap::{Args, ValueEnum};

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

async fn run_stdio(registry: ToolRegistry) -> Result<ExitCode> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    tracing::info!(
        services = registry.services().len(),
        "lab serve (stdio) ready"
    );

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();
    let mut stdout = tokio::io::stdout();

    while let Some(line) = reader.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let req: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let err = serde_json::json!({ "kind": "decode_error", "message": e.to_string() });
                stdout.write_all(err.to_string().as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                continue;
            }
        };

        let service = req.get("service").and_then(|v| v.as_str()).unwrap_or("");
        let action = req.get("action").and_then(|v| v.as_str()).unwrap_or("");
        let params = req
            .get("params")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        let result = dispatch(&registry, service, action, params).await;
        let body = match result {
            Ok(v) => serde_json::json!({ "data": v }),
            Err(e) => serde_json::json!({ "kind": "internal_error", "message": e.to_string() }),
        };
        stdout.write_all(body.to_string().as_bytes()).await?;
        stdout.write_all(b"\n").await?;
    }

    Ok(ExitCode::SUCCESS)
}

async fn dispatch(
    registry: &ToolRegistry,
    service: &str,
    action: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value> {
    if !registry.services().iter().any(|s| s.name == service) {
        anyhow::bail!("unknown service `{service}`");
    }
    match service {
        "extract" => crate::mcp::services::extract::dispatch(action, params).await,
        #[cfg(feature = "radarr")]
        "radarr" => crate::mcp::services::radarr::dispatch(action, params).await,
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
