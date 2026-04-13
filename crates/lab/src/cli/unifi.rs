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

    /// Print what would be done without executing.
    #[arg(long)]
    pub dry_run: bool,
}

/// Run the `lab unifi` subcommand.
///
/// # Errors
/// Returns an error if the client is not configured or the API call fails.
pub async fn run(args: UnifiArgs, format: OutputFormat) -> Result<ExitCode> {
    let mut params = parse_kv_params(args.params)?;
    if let Some(instance) = args.instance {
        if let serde_json::Value::Object(ref mut map) = params {
            map.insert("instance".to_string(), serde_json::Value::String(instance));
        } else {
            anyhow::bail!("--instance requires params to be key=value pairs that form a JSON object");
        }
    }
    if args.dry_run {
        match format {
            OutputFormat::Json => {
                let obj = serde_json::json!({
                    "dry_run": true,
                    "action": args.action,
                    "params": params,
                });
                println!("{}", serde_json::to_string_pretty(&obj).unwrap_or_default());
            }
            OutputFormat::Human => {
                println!(
                    "[dry-run] would dispatch unifi action `{}` with params: {}",
                    args.action,
                    serde_json::to_string(&params).unwrap_or_else(|_| "{}".to_string())
                );
            }
        }
        return Ok(ExitCode::SUCCESS);
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
