use std::fs;
use std::net::IpAddr;

use anyhow::Result;

use crate::config::{DeviceRole, ResolvedDeviceRuntime};

pub fn resolve_local_hostname() -> Result<String> {
    if let Some(value) = std::env::var("HOSTNAME")
        .ok()
        .or_else(|| std::env::var("COMPUTERNAME").ok())
        .and_then(|value| normalize_host_identifier(&value))
    {
        return Ok(value);
    }

    if let Ok(value) = fs::read_to_string("/etc/hostname") {
        if let Some(normalized) = normalize_host_identifier(&value) {
            return Ok(normalized);
        }
    }

    Ok("localhost".to_string())
}

pub fn resolve_runtime_role(
    local_host: &str,
    configured_master: Option<&str>,
) -> Result<ResolvedDeviceRuntime> {
    let local_host = normalize_host_identifier(local_host).unwrap_or_else(|| "localhost".to_string());
    let master_host = configured_master
        .and_then(normalize_host_identifier)
        .unwrap_or_else(|| local_host.clone());
    let role = if hosts_refer_to_same_device(&local_host, &master_host) {
        DeviceRole::Master
    } else {
        DeviceRole::NonMaster
    };

    Ok(ResolvedDeviceRuntime {
        local_host,
        master_host,
        role,
    })
}

fn normalize_host_identifier(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_end_matches('.');
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_ascii_lowercase())
    }
}

fn short_host_identifier(value: &str) -> &str {
    value.split('.').next().unwrap_or(value)
}

fn hosts_refer_to_same_device(local_host: &str, master_host: &str) -> bool {
    if local_host == master_host {
        return true;
    }

    let local_is_ip = local_host.parse::<IpAddr>().is_ok();
    let master_is_ip = master_host.parse::<IpAddr>().is_ok();
    if local_is_ip || master_is_ip {
        return false;
    }

    let local_short = short_host_identifier(local_host);
    let master_short = short_host_identifier(master_host);
    local_short == master_short
        && (local_host == local_short || master_host == master_short)
}
