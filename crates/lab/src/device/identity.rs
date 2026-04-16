use std::fs;

use anyhow::Result;

use crate::config::{DeviceRole, ResolvedDeviceRuntime};

pub fn resolve_local_hostname() -> Result<String> {
    if let Ok(value) = std::env::var("HOSTNAME") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    if let Ok(value) = fs::read_to_string("/etc/hostname") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    Ok("localhost".to_string())
}

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
