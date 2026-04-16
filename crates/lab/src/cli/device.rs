use std::process::ExitCode;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::config::LabConfig;
use crate::device::master_client::MasterClient;
use crate::output::{OutputFormat, print};

#[derive(Debug, Args)]
pub struct DeviceArgs {
    #[command(subcommand)]
    pub command: DeviceCommand,
}

#[derive(Debug, Subcommand)]
pub enum DeviceCommand {
    List,
    Get { device_id: String },
}

pub async fn run(args: DeviceArgs, format: OutputFormat, config: &LabConfig) -> Result<ExitCode> {
    let value = match args.command {
        DeviceCommand::List => fetch_devices(config).await?,
        DeviceCommand::Get { device_id } => fetch_device(config, &device_id).await?,
    };
    print(&value, format)?;
    Ok(ExitCode::SUCCESS)
}

pub async fn fetch_devices(config: &LabConfig) -> Result<serde_json::Value> {
    MasterClient::from_config(config)?.fetch_devices().await
}

pub async fn fetch_device(config: &LabConfig, device_id: &str) -> Result<serde_json::Value> {
    MasterClient::from_config(config)?.fetch_device(device_id).await
}
