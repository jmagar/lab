use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};

use crate::config::{DeviceRole, ResolvedDeviceRuntime};
use crate::device::checkin::{DeviceHello, DeviceMetadataUpload};
use crate::device::config_scan::discover_ai_cli_configs;
use crate::device::identity::resolve_runtime_role;
use crate::device::log_collect::collect_bootstrap_logs;
use crate::device::log_event::DeviceLogEvent;
use crate::device::master_client::MasterClient;
use crate::device::queue::{DeviceOutboundQueue, QueuedEnvelope};

#[derive(Debug, Clone)]
pub struct DeviceRuntime {
    resolved: ResolvedDeviceRuntime,
    master_client: Option<MasterClient>,
    home_dir: PathBuf,
}

impl DeviceRuntime {
    #[must_use]
    pub fn new(resolved: ResolvedDeviceRuntime, master_client: Option<MasterClient>) -> Self {
        Self {
            resolved,
            master_client,
            home_dir: current_home_dir(),
        }
    }

    #[must_use]
    pub fn for_http_master(
        resolved: ResolvedDeviceRuntime,
        master_port: u16,
    ) -> Self {
        let master_client = match resolved.role {
            DeviceRole::Master => None,
            DeviceRole::NonMaster => Some(MasterClient::new(format!(
                "http://{}:{}",
                resolved.master_host, master_port
            ))),
        };
        Self::new(resolved, master_client)
    }

    #[must_use]
    pub fn non_master_for_test(local_host: &str, base_url: String) -> Self {
        let resolved = resolve_runtime_role(local_host, Some("master"))
            .expect("non-master test runtime should resolve");
        Self::new(resolved, Some(MasterClient::new(base_url)))
    }

    #[must_use]
    pub fn non_master_for_test_with_home(
        local_host: &str,
        base_url: String,
        home_dir: &Path,
    ) -> Self {
        let mut runtime = Self::non_master_for_test(local_host, base_url);
        runtime.home_dir = home_dir.to_path_buf();
        runtime
    }

    pub async fn send_initial_hello(&self) -> Result<()> {
        if matches!(self.resolved.role, DeviceRole::Master) {
            return Ok(());
        }

        let client = self
            .master_client
            .as_ref()
            .ok_or_else(|| anyhow!("master client is not configured"))?;
        client
            .post_hello(&DeviceHello {
                device_id: self.resolved.local_host.clone(),
                role: "non-master".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            })
            .await
    }

    pub async fn upload_initial_metadata(&self) -> Result<()> {
        if matches!(self.resolved.role, DeviceRole::Master) {
            return Ok(());
        }

        let client = self
            .master_client
            .as_ref()
            .ok_or_else(|| anyhow!("master client is not configured"))?;
        let payload = DeviceMetadataUpload {
            device_id: self.resolved.local_host.clone(),
            discovered_configs: discover_ai_cli_configs(&self.home_dir)?,
        };
        client.post_metadata(&payload).await
    }

    pub async fn queue_syslog_batch(&self, events: Vec<DeviceLogEvent>) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        let queue = DeviceOutboundQueue::open(self.queue_path())
            .await
            .context("open device outbound queue")?;
        let payload = serde_json::json!({
            "device_id": self.resolved.local_host.clone(),
            "events": events,
        });
        queue.push(QueuedEnvelope::syslog_batch(payload)).await
    }

    pub async fn flush_queue_once(&self) -> Result<()> {
        if matches!(self.resolved.role, DeviceRole::Master) {
            return Ok(());
        }

        let client = self
            .master_client
            .as_ref()
            .ok_or_else(|| anyhow!("master client is not configured"))?;
        let queue = DeviceOutboundQueue::open(self.queue_path())
            .await
            .context("open device outbound queue")?;
        let drained = queue.drain_batch(100).await?;
        let mut ack_count = 0usize;

        for envelope in drained {
            match envelope.kind.as_str() {
                "syslog_batch" => {
                    client.post_syslog_batch(&envelope.payload).await?;
                    ack_count += 1;
                }
                "status" => {
                    let status = serde_json::from_value(envelope.payload)
                        .context("decode queued status envelope")?;
                    client.post_status(&status).await?;
                    ack_count += 1;
                }
                _ => break,
            }
        }

        if ack_count > 0 {
            queue.ack_drained(ack_count).await?;
        }
        Ok(())
    }

    pub async fn collect_and_flush_bootstrap_logs(&self) -> Result<()> {
        let events = collect_bootstrap_logs(&self.resolved.local_host)?;
        self.queue_syslog_batch(events).await?;
        self.flush_queue_once().await
    }

    #[must_use]
    pub const fn role(&self) -> DeviceRole {
        self.resolved.role
    }
}

fn current_home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

impl DeviceRuntime {
    fn queue_path(&self) -> PathBuf {
        self.home_dir.join(".lab/device-runtime-queue.jsonl")
    }
}
