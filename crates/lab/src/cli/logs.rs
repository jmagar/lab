use std::process::ExitCode;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::config::LabConfig;
use crate::device::master_client::MasterClient;
use crate::output::{OutputFormat, print};

#[derive(Debug, Args)]
pub struct LogsArgs {
    #[command(subcommand)]
    pub command: LogsCommand,
}

#[derive(Debug, Subcommand)]
pub enum LogsCommand {
    Search { device: String, query: String },
}

pub async fn run(args: LogsArgs, format: OutputFormat, config: &LabConfig) -> Result<ExitCode> {
    let value = match args.command {
        LogsCommand::Search { device, query } => search_logs(config, &device, &query).await?,
    };
    print(&value, format)?;
    Ok(ExitCode::SUCCESS)
}

pub async fn search_logs(
    config: &LabConfig,
    device_id: &str,
    query: &str,
) -> Result<serde_json::Value> {
    MasterClient::from_config(config)?
        .search_logs(device_id, query)
        .await
}
