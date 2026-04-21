use std::process::ExitCode;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::config::LabConfig;
use crate::device::checkin::DeviceHello;
use crate::device::identity::resolve_local_hostname;
use crate::device::master_client::MasterClient;
use crate::output::{OutputFormat, print};

#[derive(Debug, Args)]
pub struct DeviceArgs {
    #[command(subcommand)]
    pub command: DeviceCommand,
}

#[derive(Debug, Subcommand)]
pub enum DeviceCommand {
    /// List all registered devices visible from the master.
    List,
    /// Get details for a specific device by `device_id`.
    Get { device_id: String },
    /// Register this device with the master (phone home).
    Hello {
        /// Master base URL (e.g. http://dookie:8765). Defaults to config.
        #[arg(long)]
        master: Option<String>,
        /// Override the device ID sent to the master. Defaults to local hostname.
        #[arg(long)]
        device_id: Option<String>,
    },
}

pub async fn run(args: DeviceArgs, format: OutputFormat, config: &LabConfig) -> Result<ExitCode> {
    match args.command {
        DeviceCommand::List => {
            print(&fetch_devices(config).await?, format)?;
        }
        DeviceCommand::Get { device_id } => {
            print(&fetch_device(config, &device_id).await?, format)?;
        }
        DeviceCommand::Hello { master, device_id } => {
            send_hello(config, master, device_id).await?;
        }
    }
    Ok(ExitCode::SUCCESS)
}

pub async fn send_hello(
    config: &LabConfig,
    master: Option<String>,
    device_id: Option<String>,
) -> Result<()> {
    let client = match master {
        Some(url) => MasterClient::with_bearer_token(url, None)?,
        None => MasterClient::from_config(config, None)?,
    };
    let payload = DeviceHello {
        device_id: device_id.unwrap_or_else(|| {
            resolve_local_hostname().unwrap_or_else(|_| "unknown".to_string())
        }),
        role: "non-master".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    client.post_hello(&payload).await
}

pub async fn fetch_devices(config: &LabConfig) -> Result<serde_json::Value> {
    MasterClient::from_config(config, None)?
        .fetch_devices()
        .await
}

pub async fn fetch_device(config: &LabConfig, device_id: &str) -> Result<serde_json::Value> {
    MasterClient::from_config(config, None)?
        .fetch_device(device_id)
        .await
}
