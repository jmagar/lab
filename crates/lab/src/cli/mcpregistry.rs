//! `lab mcpregistry` — CLI shim for the MCP Registry service.

use std::process::ExitCode;

use anyhow::Result;
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
    #[arg(value_parser = clap::builder::PossibleValuesParser::new(ACTIONS.iter().map(|a| a.name)))]
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

async fn run_install(args: InstallArgs, format: OutputFormat) -> Result<ExitCode> {
    run_confirmable_action_command(
        "mcpregistry",
        ACTIONS,
        "server.install".to_string(),
        serde_json::json!({
            "name": args.name,
            "gateway_name": args.gateway_name,
            "bearer_token_env": args.bearer_token_env,
            "version": args.version,
        }),
        args.yes,
        format,
        |action, params| async move {
            crate::dispatch::mcpregistry::dispatch(&action, params).await
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use crate::dispatch::mcpregistry::validate_registry_url;

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
}
