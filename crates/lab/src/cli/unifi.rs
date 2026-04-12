//! `lab unifi` — CLI shim for the `UniFi` service.
//!
//! Thin shim: parse action + key/value params, call the shared dispatcher,
//! and format the result. This mirrors the MCP action surface directly.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use crate::cli::helpers::run_confirmable_action_command;
use crate::cli::params::parse_kv_params;
use crate::dispatch::unifi::actions as unifi_actions;
use crate::output::OutputFormat;

/// `lab unifi` arguments.
#[derive(Debug, Args)]
pub struct UnifiArgs {
    /// Action to run, e.g. `help`, `sites.list`, `firewall.zones.list`.
    pub action: String,

    /// Optional named instance label.
    #[arg(long)]
    pub instance: Option<String>,

    /// Optional `key=value` params for the action.
    #[arg(value_name = "KEY=VALUE", trailing_var_arg = true)]
    pub params: Vec<String>,

    /// Skip confirmation for destructive actions.
    #[arg(short = 'y', long, alias = "no-confirm")]
    pub yes: bool,
}

/// Run the `lab unifi` subcommand.
///
/// # Errors
/// Returns an error if the client is not configured or the API call fails.
pub async fn run(args: UnifiArgs, format: OutputFormat) -> Result<ExitCode> {
    let mut params = parse_kv_params(args.params)?;
    if let Some(instance) = args.instance
        && let serde_json::Value::Object(ref mut map) = params
    {
        map.insert("instance".to_string(), serde_json::Value::String(instance));
    }
    run_confirmable_action_command(
        "unifi",
        unifi_actions(),
        args.action,
        params,
        args.yes,
        format,
        |action, params| async move { crate::dispatch::unifi::dispatch(&action, params).await },
    )
    .await
}
