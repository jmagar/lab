use anyhow::Result;

use crate::config::{DeviceRole, ResolvedDeviceRuntime};

pub fn resolve_runtime_role(
    local_host: &str,
    configured_master: Option<&str>,
) -> Result<ResolvedDeviceRuntime> {
    let master_host = configured_master.unwrap_or(local_host).to_string();
    let role = if master_host == local_host {
        DeviceRole::Master
    } else {
        DeviceRole::NonMaster
    };

    Ok(ResolvedDeviceRuntime {
        local_host: local_host.to_string(),
        master_host,
        role,
    })
}
