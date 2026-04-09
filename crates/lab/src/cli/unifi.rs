//! `lab unifi` — CLI shim for the `UniFi` service.
//!
//! Thin shim: parse action + key/value params, call the shared dispatcher,
//! and format the result. This mirrors the MCP action surface directly.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::output::{OutputFormat, print};
use crate::cli::params::parse_kv_params;

/// `lab unifi` arguments.
#[derive(Debug, Args)]
pub struct UnifiArgs {
    /// Action to run, e.g. `help`, `sites.list`, `firewall.zones.list`.
    pub action: String,

    /// Optional `key=value` params for the action.
    #[arg(value_name = "KEY=VALUE", trailing_var_arg = true)]
    pub params: Vec<String>,
}

/// Run the `lab unifi` subcommand.
///
/// # Errors
/// Returns an error if the client is not configured or the API call fails.
pub async fn run(args: UnifiArgs, format: OutputFormat) -> Result<ExitCode> {
    let params = parse_kv_params(args.params)?;
    let result = crate::mcp::services::unifi::dispatch(&args.action, params)
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "{}",
                serde_json::to_string(&e).unwrap_or_else(|_| e.to_string())
            )
        })?;
    print(&result, format)?;
    Ok(ExitCode::SUCCESS)
}


