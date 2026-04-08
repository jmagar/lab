//! `lab unifi` — CLI shim for the `UniFi` service.
//!
//! Thin shim: parse → call client → format. No business logic here.

use std::process::ExitCode;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::output::{OutputFormat, print};

/// `lab unifi` arguments.
#[derive(Debug, Args)]
pub struct UnifiArgs {
    #[command(subcommand)]
    pub command: UnifiCommand,
}

/// `UniFi` subcommands.
#[derive(Debug, Subcommand)]
pub enum UnifiCommand {
    /// Return `UniFi` application version and runtime metadata.
    Info,
    /// List local `UniFi` sites.
    SitesList,
    /// List adopted devices for one site.
    DevicesList {
        /// Site UUID.
        site_id: String,
    },
    /// Inspect one adopted device.
    DeviceGet {
        /// Site UUID.
        site_id: String,
        /// Device UUID.
        device_id: String,
    },
    /// Get latest statistics for one adopted device.
    DeviceStats {
        /// Site UUID.
        site_id: String,
        /// Device UUID.
        device_id: String,
    },
    /// List devices pending adoption.
    PendingDevicesList,
    /// List connected clients for one site.
    ClientsList {
        /// Site UUID.
        site_id: String,
    },
    /// Inspect one connected client.
    ClientGet {
        /// Site UUID.
        site_id: String,
        /// Client UUID.
        client_id: String,
    },
    /// List networks for one site.
    NetworksList {
        /// Site UUID.
        site_id: String,
    },
    /// Inspect one network.
    NetworkGet {
        /// Site UUID.
        site_id: String,
        /// Network UUID.
        network_id: String,
    },
    /// Get references for one network.
    NetworkReferences {
        /// Site UUID.
        site_id: String,
        /// Network UUID.
        network_id: String,
    },
    /// List `WiFi` broadcasts for one site.
    WifiBroadcastsList {
        /// Site UUID.
        site_id: String,
    },
    /// Inspect one `WiFi` broadcast.
    WifiBroadcastGet {
        /// Site UUID.
        site_id: String,
        /// `WiFi` broadcast UUID.
        wifi_broadcast_id: String,
    },
}

/// Run the `lab unifi` subcommand.
///
/// # Errors
/// Returns an error if the client is not configured or the API call fails.
pub async fn run(args: UnifiArgs, format: OutputFormat) -> Result<ExitCode> {
    let client = crate::mcp::services::unifi::client_from_env()
        .ok_or_else(|| anyhow::anyhow!("UNIFI_URL and UNIFI_API_KEY must be set"))?;

    match args.command {
        UnifiCommand::Info => {
            print(&client.info().await?, format)?;
        }
        UnifiCommand::SitesList => {
            print(&client.sites_list().await?, format)?;
        }
        UnifiCommand::DevicesList { site_id } => {
            print(&client.devices_list(&site_id).await?, format)?;
        }
        UnifiCommand::DeviceGet { site_id, device_id } => {
            print(&client.device_get(&site_id, &device_id).await?, format)?;
        }
        UnifiCommand::DeviceStats { site_id, device_id } => {
            print(
                &client.device_stats_latest(&site_id, &device_id).await?,
                format,
            )?;
        }
        UnifiCommand::PendingDevicesList => {
            print(&client.pending_devices_list().await?, format)?;
        }
        UnifiCommand::ClientsList { site_id } => {
            print(&client.clients_list(&site_id).await?, format)?;
        }
        UnifiCommand::ClientGet { site_id, client_id } => {
            print(&client.client_get(&site_id, &client_id).await?, format)?;
        }
        UnifiCommand::NetworksList { site_id } => {
            print(&client.networks_list(&site_id).await?, format)?;
        }
        UnifiCommand::NetworkGet {
            site_id,
            network_id,
        } => {
            print(&client.network_get(&site_id, &network_id).await?, format)?;
        }
        UnifiCommand::NetworkReferences {
            site_id,
            network_id,
        } => {
            print(
                &client.network_references(&site_id, &network_id).await?,
                format,
            )?;
        }
        UnifiCommand::WifiBroadcastsList { site_id } => {
            print(&client.wifi_broadcasts_list(&site_id).await?, format)?;
        }
        UnifiCommand::WifiBroadcastGet {
            site_id,
            wifi_broadcast_id,
        } => {
            print(
                &client
                    .wifi_broadcast_get(&site_id, &wifi_broadcast_id)
                    .await?,
                format,
            )?;
        }
    }

    Ok(ExitCode::SUCCESS)
}
