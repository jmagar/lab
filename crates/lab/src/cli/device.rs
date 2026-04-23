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
    /// List all registered devices visible from the master.
    List,
    /// Get details for a specific device by `device_id`.
    Get { device_id: String },
    /// Manage pending, approved, and denied device enrollments.
    Enrollments(EnrollmentArgs),
}

#[derive(Debug, Args)]
pub struct EnrollmentArgs {
    #[command(subcommand)]
    pub command: EnrollmentCommand,
}

#[derive(Debug, Subcommand)]
pub enum EnrollmentCommand {
    /// List pending, approved, and denied enrollments.
    List,
    /// Approve a pending enrollment.
    Approve {
        device_id: String,
        #[arg(long)]
        note: Option<String>,
    },
    /// Deny a pending or approved enrollment.
    Deny {
        device_id: String,
        #[arg(long)]
        reason: Option<String>,
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
        DeviceCommand::Enrollments(args) => match args.command {
            EnrollmentCommand::List => {
                print(&fetch_enrollments(config).await?, format)?;
            }
            EnrollmentCommand::Approve { device_id, note } => {
                print(&approve_enrollment(config, &device_id, note.as_deref()).await?, format)?;
            }
            EnrollmentCommand::Deny { device_id, reason } => {
                print(&deny_enrollment(config, &device_id, reason.as_deref()).await?, format)?;
            }
        },
    }
    Ok(ExitCode::SUCCESS)
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

pub async fn fetch_enrollments(config: &LabConfig) -> Result<serde_json::Value> {
    MasterClient::from_config(config, None)?
        .fetch_enrollments()
        .await
}

pub async fn approve_enrollment(
    config: &LabConfig,
    device_id: &str,
    note: Option<&str>,
) -> Result<serde_json::Value> {
    MasterClient::from_config(config, None)?
        .approve_enrollment(device_id, note)
        .await
}

pub async fn deny_enrollment(
    config: &LabConfig,
    device_id: &str,
    reason: Option<&str>,
) -> Result<serde_json::Value> {
    MasterClient::from_config(config, None)?
        .deny_enrollment(device_id, reason)
        .await
}
