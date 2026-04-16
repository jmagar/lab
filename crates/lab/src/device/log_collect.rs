use anyhow::Result;

use crate::device::log_event::DeviceLogEvent;

pub fn collect_bootstrap_logs(_device_id: &str) -> Result<Vec<DeviceLogEvent>> {
    Ok(Vec::new())
}
