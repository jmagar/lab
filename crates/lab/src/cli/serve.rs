//! `lab serve` — start the MCP server.

use std::process::ExitCode;

use anyhow::Result;
use clap::{Args, ValueEnum};

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
    /// Comma- or space-separated list of services to enable.
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

/// Run the serve subcommand. Stub — real rmcp wiring comes in a later plan.
pub async fn run(args: ServeArgs) -> Result<ExitCode> {
    tracing::warn!(
        services = ?args.services,
        transport = ?args.transport,
        host = %args.host,
        port = args.port,
        "lab serve: MCP server not yet wired — skeleton stub",
    );
    Ok(ExitCode::SUCCESS)
}
