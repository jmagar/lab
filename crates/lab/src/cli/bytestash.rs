//! `lab bytestash` — CLI shim for the `ByteStash` service.
//!
//! Thin shim: parse action + key/value params, call the shared dispatcher,
//! and format the result. This mirrors the MCP action surface directly.

use std::process::ExitCode;

use anyhow::Result;
use clap::Args;
use serde_json::{Map, Value};

use crate::output::{OutputFormat, print};

/// `lab bytestash` arguments.
#[derive(Debug, Args)]
pub struct BytestashArgs {
    /// Action to run, e.g. `help`, `snippets.list`, `categories.list`.
    pub action: String,

    /// Optional `key=value` params for the action.
    #[arg(value_name = "KEY=VALUE", trailing_var_arg = true)]
    pub params: Vec<String>,
}

/// Run the `lab bytestash` subcommand.
///
/// # Errors
/// Returns an error if the client is not configured or the API call fails.
pub async fn run(args: BytestashArgs, format: OutputFormat) -> Result<ExitCode> {
    let params = parse_params(args.params)?;
    let result = crate::mcp::services::bytestash::dispatch(&args.action, params)
        .await
        .map_err(|te| {
            anyhow::anyhow!(
                "{}",
                serde_json::to_string(&te).unwrap_or_else(|_| format!("{te:?}"))
            )
        })?;
    print(&result, format)?;
    Ok(ExitCode::SUCCESS)
}

fn parse_params(params: Vec<String>) -> Result<Value> {
    let mut map = Map::new();
    for item in params {
        let Some((key, raw)) = item.split_once('=') else {
            anyhow::bail!("invalid param `{item}`; expected key=value");
        };
        map.insert(key.to_string(), coerce_value(raw));
    }
    Ok(Value::Object(map))
}

fn coerce_value(raw: &str) -> Value {
    if raw.eq_ignore_ascii_case("true") {
        return Value::Bool(true);
    }
    if raw.eq_ignore_ascii_case("false") {
        return Value::Bool(false);
    }
    if let Ok(n) = raw.parse::<i64>() {
        return Value::Number(n.into());
    }
    if let Ok(n) = raw.parse::<f64>()
        && let Some(num) = serde_json::Number::from_f64(n)
    {
        return Value::Number(num);
    }
    Value::String(raw.to_string())
}
