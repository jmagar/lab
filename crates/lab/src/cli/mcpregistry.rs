//! `lab mcpregistry` — CLI shim for the MCP Registry service.

use std::net::ToSocketAddrs;
use std::process::ExitCode;

use anyhow::{Context, Result, anyhow};
use clap::{Args, Subcommand};

use crate::cli::helpers::run_confirmable_action_command;
use crate::cli::params::parse_kv_params;
use crate::dispatch::mcpregistry::ACTIONS;
use crate::output::OutputFormat;

/// `lab mcpregistry` arguments.
#[derive(Debug, Args)]
pub struct McpregistryArgs {
    #[command(subcommand)]
    pub command: McpregistryCommand,
}

#[derive(Debug, Subcommand)]
pub enum McpregistryCommand {
    /// Dispatch an action directly (e.g. `help`, `server.list`, `server.get`).
    Action(ActionArgs),
    /// Install an MCP server from the registry as a gateway upstream.
    Install(InstallArgs),
}

/// Arguments for the generic `action` subcommand.
#[derive(Debug, Args)]
pub struct ActionArgs {
    /// Action to run, e.g. `help`, `server.list`, `server.get`.
    pub action: String,

    /// Optional `key=value` params for the action.
    #[arg(value_name = "KEY=VALUE", trailing_var_arg = true)]
    pub params: Vec<String>,

    /// Skip confirmation for destructive actions.
    #[arg(short = 'y', long, alias = "no-confirm")]
    pub yes: bool,

    /// Print what would be done without executing.
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for `lab mcpregistry install`.
#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Registry server name (e.g. `io.github.user/my-mcp`).
    pub name: String,

    /// Name to use in the gateway config. Defaults to the segment after the last `/`.
    #[arg(long)]
    pub gateway_name: Option<String>,

    /// Name of the env var holding the bearer token (not the token value itself).
    #[arg(long)]
    pub bearer_token_env: Option<String>,

    /// Registry version to fetch. Defaults to `latest`.
    #[arg(long, default_value = "latest")]
    pub version: String,

    /// Skip the confirmation prompt (gateway.add is destructive).
    #[arg(short = 'y', long, alias = "no-confirm")]
    pub yes: bool,
}

/// Run the `lab mcpregistry` subcommand.
///
/// # Errors
/// Returns an error if the client cannot be initialized or the API call fails.
pub async fn run(args: McpregistryArgs, format: OutputFormat) -> Result<ExitCode> {
    match args.command {
        McpregistryCommand::Action(a) => run_action(a, format).await,
        McpregistryCommand::Install(a) => run_install(a, format).await,
    }
}

#[allow(clippy::print_stdout)]
async fn run_action(args: ActionArgs, format: OutputFormat) -> Result<ExitCode> {
    let params = parse_kv_params(args.params)?;
    if args.dry_run {
        println!(
            "[dry-run] would dispatch mcpregistry action `{}` with params: {}",
            args.action,
            serde_json::to_string(&params).unwrap_or_else(|_| "{}".to_string())
        );
        return Ok(ExitCode::SUCCESS);
    }
    run_confirmable_action_command(
        "mcpregistry",
        ACTIONS,
        args.action,
        params,
        args.yes,
        format,
        |action, params| async move { crate::dispatch::mcpregistry::dispatch(&action, params).await },
    )
    .await
}

#[allow(clippy::print_stdout)]
async fn run_install(args: InstallArgs, format: OutputFormat) -> Result<ExitCode> {
    // Step 1: fetch server from registry.
    let get_result = crate::dispatch::mcpregistry::dispatch(
        "server.get",
        serde_json::json!({ "name": args.name }),
    )
    .await
    .map_err(|e| anyhow!("registry lookup failed: {e}"))?;

    // Step 2: extract remotes[0].url.
    let url = get_result
        .pointer("/server/remotes/0/url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            anyhow!(
                "server '{}' has no remote transport URLs — cannot install as HTTP gateway.\n\
                 Tip: check `lab mcpregistry action server.get name={}` for package-only servers.",
                args.name,
                args.name
            )
        })?
        .to_string();

    // Step 3: SSRF validation before passing to gateway.add.
    // DNS TOCTOU: validated at call time; short-TTL rebind could bypass. Homelab threat model accepts this.
    let url_for_check = url.clone();
    tokio::task::spawn_blocking(move || validate_registry_url(&url_for_check))
        .await
        .context("SSRF validation task panicked")?
        .context("registry URL failed SSRF validation")?;

    // Step 4: derive gateway name (last segment after `/`).
    let gateway_name = args.gateway_name.unwrap_or_else(|| {
        args.name
            .rsplit('/')
            .next()
            .unwrap_or(&args.name)
            .to_string()
    });

    let spec = serde_json::json!({
        "name": gateway_name,
        "url": url,
        "bearer_token_env": args.bearer_token_env,
        "command": null,
        "args": [],
        "proxy_resources": false,
        "expose_tools": null,
    });
    let params = serde_json::json!({ "spec": spec });

    // Step 5: delegate to gateway.add (destructive; honors -y).
    let gateway_actions = crate::dispatch::gateway::ACTIONS;
    run_confirmable_action_command(
        "gateway",
        gateway_actions,
        "gateway.add".to_string(),
        params,
        args.yes,
        format,
        |action, p| async move { crate::dispatch::gateway::dispatch(&action, p).await },
    )
    .await
    .context("gateway.add failed")?;

    println!("Installed: {gateway_name} → {url}");
    Ok(ExitCode::SUCCESS)
}

/// Validate that a registry-sourced URL is safe to install as a gateway upstream.
///
/// Rejects non-HTTPS schemes and hosts that resolve to private/loopback ranges.
///
/// # MUST NOT call from async context without spawn_blocking
/// Uses synchronous `ToSocketAddrs` DNS resolution.
fn validate_registry_url(url: &str) -> Result<()> {
    let parsed = url::Url::parse(url).with_context(|| format!("invalid URL: {url}"))?;

    if parsed.scheme() != "https" {
        return Err(anyhow!(
            "registry URL must use HTTPS, got '{}': {url}",
            parsed.scheme()
        ));
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| anyhow!("URL has no host: {url}"))?;
    let port = parsed.port_or_known_default().unwrap_or(443);

    // Check literal IP addresses directly without DNS.
    if let Ok(addr) = host.parse::<std::net::IpAddr>() {
        return check_ip_not_private(addr, url);
    }

    // Resolve hostname; reject if any resolved address is private.
    // Synchronous DNS call — MUST NOT be called from async without spawn_blocking.
    let addrs: Vec<_> = format!("{host}:{port}")
        .to_socket_addrs()
        .with_context(|| format!("failed to resolve host '{host}'"))?
        .collect();

    for sock_addr in addrs {
        check_ip_not_private(sock_addr.ip(), url)?;
    }

    Ok(())
}

fn check_ip_not_private(ip: std::net::IpAddr, url: &str) -> Result<()> {
    let blocked = match ip {
        std::net::IpAddr::V4(v4) => {
            let o = v4.octets();
            v4.is_loopback()                                              // 127.0.0.0/8
                || o[0] == 10                                             // 10.0.0.0/8
                || (o[0] == 172 && o[1] >= 16 && o[1] <= 31)            // 172.16.0.0/12
                || (o[0] == 192 && o[1] == 168)                          // 192.168.0.0/16
                || (o[0] == 169 && o[1] == 254) // 169.254.0.0/16 link-local / IMDS
        }
        std::net::IpAddr::V6(v6) => v6.is_loopback(), // ::1/128
    };
    if blocked {
        return Err(anyhow!(
            "registry URL resolves to a private/loopback address — blocked to prevent SSRF: {url}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_registry_url_blocks_http() {
        let err = validate_registry_url("http://example.com/mcp").unwrap_err();
        assert!(err.to_string().contains("HTTPS"), "{err}");
    }

    #[test]
    fn validate_registry_url_blocks_loopback_ip() {
        let err = validate_registry_url("https://127.0.0.1/mcp").unwrap_err();
        assert!(err.to_string().contains("private"), "{err}");
    }

    #[test]
    fn validate_registry_url_blocks_rfc1918_10() {
        let err = validate_registry_url("https://10.0.0.1/mcp").unwrap_err();
        assert!(err.to_string().contains("private"), "{err}");
    }

    #[test]
    fn validate_registry_url_blocks_rfc1918_192_168() {
        let err = validate_registry_url("https://192.168.1.1/mcp").unwrap_err();
        assert!(err.to_string().contains("private"), "{err}");
    }

    #[test]
    fn validate_registry_url_blocks_rfc1918_172_16() {
        let err = validate_registry_url("https://172.16.0.1/mcp").unwrap_err();
        assert!(err.to_string().contains("private"), "{err}");
    }

    #[test]
    fn validate_registry_url_blocks_link_local() {
        let err = validate_registry_url("https://169.254.169.254/mcp").unwrap_err();
        assert!(err.to_string().contains("private"), "{err}");
    }

    #[test]
    fn validate_registry_url_blocks_ipv6_loopback() {
        let err = validate_registry_url("https://[::1]/mcp").unwrap_err();
        assert!(err.to_string().contains("private"), "{err}");
    }

    #[test]
    fn validate_registry_url_rejects_missing_scheme() {
        validate_registry_url("not-a-url").unwrap_err();
    }

    #[test]
    fn install_default_gateway_name_last_segment() {
        let name = "io.github.user/my-mcp";
        let gateway_name = name.rsplit('/').next().unwrap_or(name).to_string();
        assert_eq!(gateway_name, "my-mcp");
    }

    #[test]
    fn install_default_gateway_name_no_slash() {
        let name = "my-server";
        let gateway_name = name.rsplit('/').next().unwrap_or(name).to_string();
        assert_eq!(gateway_name, "my-server");
    }
}
