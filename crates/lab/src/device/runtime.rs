use anyhow::{Result, anyhow};

use crate::config::{DeviceRole, ResolvedDeviceRuntime};
use crate::device::checkin::DeviceHello;
use crate::device::identity::resolve_runtime_role;
use crate::device::master_client::MasterClient;

#[derive(Debug, Clone)]
pub struct DeviceRuntime {
    resolved: ResolvedDeviceRuntime,
    master_client: Option<MasterClient>,
}

impl DeviceRuntime {
    #[must_use]
    pub fn new(resolved: ResolvedDeviceRuntime, master_client: Option<MasterClient>) -> Self {
        Self {
            resolved,
            master_client,
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
}
