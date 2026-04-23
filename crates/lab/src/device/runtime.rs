use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use tokio::sync::OnceCell;

use crate::config::{DeviceRole, LabConfig, ResolvedDeviceRuntime};
use crate::device::checkin::DeviceMetadataUpload;
use crate::device::config_scan::discover_ai_cli_configs;
use crate::device::identity::resolve_runtime_role;
use crate::device::log_collect::collect_bootstrap_logs;
use crate::device::log_event::DeviceLogEvent;
use crate::device::master_client::MasterClient;
use crate::device::queue::{DeviceOutboundQueue, QueuedEnvelope};
use crate::device::ws_client::WsClient;

#[derive(Debug, Clone)]
pub struct DeviceRuntime {
    resolved: ResolvedDeviceRuntime,
    master_client: Option<MasterClient>,
    home_dir: PathBuf,
    outbound_queue: Arc<OnceCell<DeviceOutboundQueue>>,
}

impl DeviceRuntime {
    #[must_use]
    pub fn new(resolved: ResolvedDeviceRuntime, master_client: Option<MasterClient>) -> Self {
        Self {
            resolved,
            master_client,
            home_dir: current_home_dir(),
            outbound_queue: Arc::new(OnceCell::new()),
        }
    }

    pub fn from_config(
        resolved: ResolvedDeviceRuntime,
        config: &LabConfig,
        master_port_override: Option<u16>,
    ) -> Result<Self> {
        let master_client = match resolved.role {
            DeviceRole::Master => None,
            DeviceRole::NonMaster => Some(MasterClient::from_config(config, master_port_override)?),
        };
        Ok(Self::new(resolved, master_client))
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn non_master_for_test(local_host: &str, base_url: String) -> Result<Self> {
        let resolved = resolve_runtime_role(local_host, Some("master"))?;
        Ok(Self::new(resolved, Some(MasterClient::new(base_url)?)))
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn non_master_for_test_with_home(
        local_host: &str,
        base_url: String,
        home_dir: &Path,
    ) -> Result<Self> {
        let mut runtime = Self::non_master_for_test(local_host, base_url)?;
        runtime.home_dir = home_dir.to_path_buf();
        Ok(runtime)
    }

    pub async fn upload_initial_metadata(&self) -> Result<()> {
        if matches!(self.resolved.role, DeviceRole::Master) {
            return Ok(());
        }

        let payload = DeviceMetadataUpload {
            device_id: self.resolved.local_host.clone(),
            discovered_configs: discover_ai_cli_configs(&self.home_dir)?,
        };
        let queue = self.outbound_queue().await?;
        queue
            .push(QueuedEnvelope::metadata(serde_json::to_value(payload)?))
            .await
    }

    pub async fn queue_syslog_batch(&self, events: Vec<DeviceLogEvent>) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        let queue = self.outbound_queue().await?;
        let payload = serde_json::json!({
            "device_id": self.resolved.local_host.clone(),
            "events": events,
        });
        queue.push(QueuedEnvelope::syslog_batch(payload)).await
    }

    pub async fn collect_and_flush_bootstrap_logs(&self) -> Result<()> {
        if matches!(self.resolved.role, DeviceRole::Master) {
            return Ok(());
        }

        let events = collect_bootstrap_logs(&self.resolved.local_host)?;
        self.queue_syslog_batch(events).await
    }

    pub async fn spawn_ws_flush_loop(&self) -> Result<()> {
        if matches!(self.resolved.role, DeviceRole::Master) {
            return Ok(());
        }

        let client = self
            .master_client
            .as_ref()
            .ok_or_else(|| anyhow!("master client is not configured"))?;
        let ws_client = WsClient::new(
            client.base_url(),
            self.resolved.local_host.clone(),
            self.device_token_path(),
        )?;
        let queue = Arc::new(self.outbound_queue().await?.clone());
        tokio::spawn(async move {
            ws_client.run(queue).await;
        });
        Ok(())
    }

    #[must_use]
    pub const fn role(&self) -> DeviceRole {
        self.resolved.role
    }

    #[must_use]
    pub fn home_dir(&self) -> &Path {
        &self.home_dir
    }
}

fn current_home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .or_else(|| {
            let home_drive = std::env::var_os("HOMEDRIVE")?;
            let home_path = std::env::var_os("HOMEPATH")?;
            let mut path = PathBuf::from(home_drive);
            path.push(home_path);
            Some(path.into_os_string())
        })
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}

impl DeviceRuntime {
    fn queue_path(&self) -> PathBuf {
        self.home_dir.join(".lab/device-runtime-queue.jsonl")
    }

    fn device_token_path(&self) -> PathBuf {
        self.home_dir.join(".lab/device-token")
    }

    async fn outbound_queue(&self) -> Result<&DeviceOutboundQueue> {
        self.outbound_queue
            .get_or_try_init(|| async {
                DeviceOutboundQueue::open(self.queue_path())
                    .await
                    .context("open device outbound queue")
            })
            .await
    }
}
