use std::process::ExitCode;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use serde_json::{Value, json};

use crate::cli::helpers::run_action_command;
use crate::config::LabConfig;
use crate::device::master_client::MasterClient;
use crate::dispatch::logs::client::{
    bootstrap_store_backed_log_system, resolve_retention, resolve_store_path,
};
use crate::dispatch::logs::dispatch::dispatch_with_system;
use crate::dispatch::logs::types::{LogQuery, LogSystem, LogTailRequest, RawLogEvent};
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
    /// Forward this node's syslog to the master log store (peer mode).
    Forward(ForwardArgs),
}

#[derive(Debug, Args)]
pub struct ForwardArgs {
    /// Override the master base URL (default: LAB_MASTER_URL).
    #[arg(long, env = "LAB_MASTER_URL")]
    pub master_url: Option<String>,
    /// Override the bearer token (default: LAB_MASTER_TOKEN or LAB_MCP_HTTP_TOKEN).
    #[arg(long, env = "LAB_MASTER_TOKEN")]
    pub master_token: Option<String>,
    /// Node ID to stamp on every forwarded event (default: LAB_NODE_ID or hostname).
    #[arg(long, env = "LAB_NODE_ID")]
    pub node_id: Option<String>,
    /// How many events to batch per request (default 200).
    #[arg(long, default_value = "200")]
    pub batch_size: usize,
    /// Skip journald and read directly from /var/log/syslog.
    #[arg(long)]
    pub syslog_only: bool,
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
        LogsCommand::Forward(args) => run_forward(args, config).await,
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
        source_node_ids: vec![],
        source_kinds: vec![],
        limit: args.limit,
    }
}

async fn run_forward(args: ForwardArgs, _config: &LabConfig) -> Result<ExitCode> {
    let master_url = args
        .master_url
        .or_else(|| {
            std::env::var("LAB_MASTER_URL")
                .ok()
                .filter(|s| !s.is_empty())
        })
        .context("LAB_MASTER_URL is not set; pass --master-url or set the env var")?;

    let token = args
        .master_token
        .or_else(|| {
            std::env::var("LAB_MASTER_TOKEN")
                .ok()
                .filter(|s| !s.is_empty())
        })
        .or_else(|| {
            std::env::var("LAB_MCP_HTTP_TOKEN")
                .ok()
                .filter(|s| !s.is_empty())
        });

    let node_id = args
        .node_id
        .or_else(|| std::env::var("LAB_NODE_ID").ok().filter(|s| !s.is_empty()))
        .unwrap_or_else(resolve_node_id);

    let client = MasterClient::with_bearer_token(master_url, token)?;

    tracing::info!(node_id = node_id.as_str(), "starting syslog forward");

    if !args.syslog_only && journald_available() {
        forward_journald(&client, &node_id, args.batch_size).await
    } else {
        forward_syslog_file(&client, &node_id, args.batch_size).await
    }
}

fn resolve_node_id() -> String {
    std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn journald_available() -> bool {
    std::process::Command::new("journalctl")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

async fn forward_journald(
    client: &MasterClient,
    node_id: &str,
    batch_size: usize,
) -> Result<ExitCode> {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};
    use tokio::sync::mpsc;
    use tokio::time::{Duration, interval};

    let mut child = Command::new("journalctl")
        // --lines=100 picks up recent history immediately before following new entries.
        .args(["--follow", "--output=json", "--lines=100"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to spawn journalctl")?;

    let stdout = child
        .stdout
        .take()
        .context("failed to capture journalctl stdout")?;

    // Blocking reader → async channel bridge so we can select! with a flush timer.
    let (tx, mut rx) = mpsc::channel::<Option<String>>(256);

    tokio::task::spawn_blocking(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    if tx.blocking_send(Some(l)).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        // Signal EOF.
        drop(tx.blocking_send(None));
    });

    let mut batch: Vec<RawLogEvent> = Vec::with_capacity(batch_size);
    let mut flush_tick = interval(Duration::from_secs(2));
    flush_tick.tick().await; // consume the immediate first tick

    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Some(Some(line)) => {
                        batch.push(parse_journald_line(&line));
                        if batch.len() >= batch_size {
                            flush_batch(client, node_id, &mut batch).await?;
                        }
                    }
                    // EOF or channel closed — flush and exit.
                    _ => {
                        if !batch.is_empty() {
                            flush_batch(client, node_id, &mut batch).await?;
                        }
                        return Ok(ExitCode::SUCCESS);
                    }
                }
            }
            _ = flush_tick.tick() => {
                if !batch.is_empty() {
                    flush_batch(client, node_id, &mut batch).await?;
                }
            }
        }
    }
}

fn parse_journald_line(line: &str) -> RawLogEvent {
    let Ok(obj) = serde_json::from_str::<Value>(line) else {
        return RawLogEvent {
            message: line.to_string(),
            source_kind: Some("syslog".to_string()),
            ..default_raw_event()
        };
    };

    let message = obj
        .get("MESSAGE")
        .and_then(|v| v.as_str())
        .unwrap_or(line)
        .to_string();

    // journald PRIORITY: 0=emerg 1=alert 2=crit 3=err 4=warning 5=notice 6=info 7=debug
    let level = obj.get("PRIORITY").and_then(|v| v.as_str()).map(|p| {
        match p {
            "0" | "1" | "2" | "3" => "error",
            "4" => "warn",
            "5" | "6" => "info",
            "7" => "debug",
            _ => "info",
        }
        .to_string()
    });

    // __REALTIME_TIMESTAMP is microseconds since epoch.
    let ts = obj
        .get("__REALTIME_TIMESTAMP")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i64>().ok())
        .map(|us| us / 1000); // convert to ms

    let upstream_event_id = obj
        .get("_MACHINE_ID")
        .zip(obj.get("__CURSOR"))
        .and_then(|(mid, cur)| Some(format!("{}:{}", mid.as_str()?, cur.as_str()?)));

    let unit = obj
        .get("_SYSTEMD_UNIT")
        .or_else(|| obj.get("UNIT"))
        .and_then(|v| v.as_str())
        .map(str::to_string);

    RawLogEvent {
        message,
        level,
        ts,
        source_kind: Some("syslog".to_string()),
        ingest_path: Some("journald".to_string()),
        upstream_event_id,
        action: unit,
        ..default_raw_event()
    }
}

async fn forward_syslog_file(
    client: &MasterClient,
    node_id: &str,
    batch_size: usize,
) -> Result<ExitCode> {
    use std::fs::File;
    use std::io::{BufRead, BufReader, Seek, SeekFrom};
    use tokio::time::{Duration, sleep};

    let path = "/var/log/syslog";
    let mut file = File::open(path).with_context(|| format!("cannot open {path}"))?;
    // Tail from the end.
    file.seek(SeekFrom::End(0))?;

    let mut reader = BufReader::new(file);
    let mut batch: Vec<RawLogEvent> = Vec::with_capacity(batch_size);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader
            .read_line(&mut line)
            .context("error reading syslog")?;
        if n == 0 {
            if !batch.is_empty() {
                flush_batch(client, node_id, &mut batch).await?;
            }
            sleep(Duration::from_millis(250)).await;
            continue;
        }

        let event = parse_syslog_line(line.trim_end());
        batch.push(event);

        if batch.len() >= batch_size {
            flush_batch(client, node_id, &mut batch).await?;
        }
    }
}

fn parse_syslog_line(line: &str) -> RawLogEvent {
    // Best-effort RFC 3164 parse: "Mon DD HH:MM:SS host tag: message"
    // We don't do strict validation — just extract what we can.
    let message = line.splitn(4, ' ').nth(3).unwrap_or(line).to_string();
    RawLogEvent {
        message,
        source_kind: Some("syslog".to_string()),
        ingest_path: Some("syslog_file".to_string()),
        ..default_raw_event()
    }
}

fn default_raw_event() -> RawLogEvent {
    RawLogEvent {
        ts: None,
        level: None,
        subsystem: Some("syslog".to_string()),
        surface: Some("core_runtime".to_string()),
        action: None,
        message: String::new(),
        request_id: None,
        session_id: None,
        correlation_id: None,
        trace_id: None,
        span_id: None,
        instance: None,
        auth_flow: None,
        outcome_kind: None,
        fields_json: Value::Object(Default::default()),
        source_kind: None,
        source_node_id: None,
        source_device_id: None,
        ingest_path: None,
        upstream_event_id: None,
    }
}

async fn flush_batch(
    client: &MasterClient,
    node_id: &str,
    batch: &mut Vec<RawLogEvent>,
) -> Result<()> {
    let payload = json!({
        "node_id": node_id,
        "events": std::mem::take(batch),
    });
    match client.post_log_ingest(&payload).await {
        Ok(resp) => {
            tracing::debug!(
                node_id,
                accepted = resp.get("accepted").and_then(|v| v.as_u64()),
                dropped = resp.get("dropped").and_then(|v| v.as_u64()),
                "flushed log batch"
            );
        }
        Err(e) => {
            tracing::warn!(node_id, error = %e, "failed to flush log batch; events lost");
        }
    }
    Ok(())
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
