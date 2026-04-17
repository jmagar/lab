use std::process::ExitCode;
use std::sync::Arc;

use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::{Value, json};

use crate::cli::helpers::run_action_command;
use crate::config::LabConfig;
use crate::device::master_client::MasterClient;
use crate::dispatch::error::ToolError;
use crate::dispatch::logs::client::{
    bootstrap_store_backed_log_system, resolve_retention, resolve_store_path,
};
use crate::dispatch::logs::dispatch::dispatch_with_system;
use crate::dispatch::logs::types::{LogQuery, LogSystem, LogTailRequest};
use crate::output::{OutputFormat, print};

#[derive(Debug, Args)]
pub struct LogsArgs {
    #[command(subcommand)]
    pub command: LogsCommand,
}

#[derive(Debug, Subcommand)]
pub enum LogsCommand {
    /// Search fleet logs for a device from the master control plane.
    Search { device: String, query: String },
    /// Search or inspect the local-master runtime log store.
    Local(LocalLogsArgs),
}

#[derive(Debug, Args)]
pub struct LocalLogsArgs {
    #[command(subcommand)]
    pub command: LocalLogsCommand,
}

#[derive(Debug, Subcommand)]
pub enum LocalLogsCommand {
    /// Search the persistent local log store.
    Search(LocalSearchArgs),
    /// Read a bounded follow-up window from the persistent local log store.
    Tail(LocalTailArgs),
    /// Inspect local retention and drop counters.
    Stats,
    /// Live push is HTTP SSE only in v1; this command fails with guidance.
    Stream,
}

#[derive(Debug, Args, Default)]
pub struct LocalSearchArgs {
    #[arg(long)]
    pub text: Option<String>,
    #[arg(long)]
    pub after_ts: Option<i64>,
    #[arg(long)]
    pub before_ts: Option<i64>,
    #[arg(long = "level")]
    pub levels: Vec<crate::dispatch::logs::types::LogLevel>,
    #[arg(long = "subsystem")]
    pub subsystems: Vec<crate::dispatch::logs::types::Subsystem>,
    #[arg(long = "surface")]
    pub surfaces: Vec<crate::dispatch::logs::types::Surface>,
    #[arg(long)]
    pub action: Option<String>,
    #[arg(long)]
    pub request_id: Option<String>,
    #[arg(long)]
    pub session_id: Option<String>,
    #[arg(long)]
    pub correlation_id: Option<String>,
    #[arg(long)]
    pub limit: Option<usize>,
}

#[derive(Debug, Args, Default)]
pub struct LocalTailArgs {
    #[arg(long)]
    pub after_ts: Option<i64>,
    #[arg(long)]
    pub since_event_id: Option<String>,
    #[arg(long)]
    pub limit: Option<usize>,
}

pub async fn run(args: LogsArgs, format: OutputFormat, config: &LabConfig) -> Result<ExitCode> {
    match args.command {
        LogsCommand::Search { device, query } => {
            let value = search_logs(config, &device, &query).await?;
            print(&value, format)?;
            Ok(ExitCode::SUCCESS)
        }
        LogsCommand::Local(local) => run_local(local, format, config).await,
    }
}

pub async fn search_logs(config: &LabConfig, device_id: &str, query: &str) -> Result<Value> {
    MasterClient::from_config(config, None)?
        .search_logs(device_id, query)
        .await
}

pub async fn run_local(
    local: LocalLogsArgs,
    format: OutputFormat,
    config: &LabConfig,
) -> Result<ExitCode> {
    let system = local_log_system(config).await?;
    let (action, params) = match local.command {
        LocalLogsCommand::Search(args) => (
            "logs.search".to_string(),
            json!({ "query": build_search_query(args) }),
        ),
        LocalLogsCommand::Tail(args) => (
            "logs.tail".to_string(),
            serde_json::to_value(LogTailRequest {
                after_ts: args.after_ts,
                since_event_id: args.since_event_id,
                limit: args.limit,
            })?,
        ),
        LocalLogsCommand::Stats => ("logs.stats".to_string(), json!({})),
        LocalLogsCommand::Stream => {
            return Err(anyhow::anyhow!(
                "true live log streaming is only available over HTTP SSE at `/v1/logs/stream`; use `lab logs local tail` for bounded follow-up windows"
            ));
        }
    };

    run_action_command("logs", action, params, format, move |action, params| {
        let system = Arc::clone(&system);
        async move { dispatch_with_system(&system, &action, params).await }
    })
    .await
}

async fn local_log_system(config: &LabConfig) -> Result<Arc<LogSystem>> {
    Ok(bootstrap_store_backed_log_system(
        resolve_store_path(Some(config)),
        resolve_retention(Some(config)),
    )
    .await?)
}

fn build_search_query(args: LocalSearchArgs) -> LogQuery {
    LogQuery {
        text: args.text,
        after_ts: args.after_ts,
        before_ts: args.before_ts,
        levels: args.levels,
        subsystems: args.subsystems,
        surfaces: args.surfaces,
        action: args.action,
        request_id: args.request_id,
        session_id: args.session_id,
        correlation_id: args.correlation_id,
        limit: args.limit,
    }
}

#[allow(dead_code)]
pub async fn run_local_search_for_test(
    system: Arc<LogSystem>,
    query: LogQuery,
) -> Result<Value, ToolError> {
    dispatch_with_system(&system, "logs.search", json!({ "query": query })).await
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;
    use clap::Parser;

    use crate::cli::{Cli, Command};

    #[test]
    fn logs_cli_parser_accepts_existing_fleet_search() {
        Cli::command().debug_assert();
        assert!(Cli::try_parse_from(["lab", "logs", "search", "node-a", "timeout"]).is_ok());
    }

    #[test]
    fn logs_cli_parses_local_search() {
        let cli = Cli::try_parse_from([
            "lab",
            "logs",
            "local",
            "search",
            "--subsystem",
            "gateway",
            "--level",
            "warn",
        ])
        .expect("local search parses");

        assert!(matches!(cli.command, Command::Logs(_)));
    }

    #[test]
    fn logs_cli_rejects_invalid_local_search_filters() {
        assert!(
            Cli::try_parse_from(["lab", "logs", "local", "search", "--level", "warning",]).is_err()
        );
    }
}
