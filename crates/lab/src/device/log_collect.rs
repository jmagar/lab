use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Result, bail};

use crate::device::log_event::DeviceLogEvent;

const BOOTSTRAP_LOG_LINE_LIMIT: usize = 128;
const BOOTSTRAP_LOG_BYTE_LIMIT: u64 = 256 * 1024;
const CANDIDATE_LOG_PATHS: &[&str] = &["/var/log/syslog", "/var/log/messages"];

pub fn collect_bootstrap_logs(device_id: &str) -> Result<Vec<DeviceLogEvent>> {
    for candidate in CANDIDATE_LOG_PATHS {
        let path = Path::new(candidate);
        let Ok(metadata) = fs::metadata(path) else {
            continue;
        };
        if !metadata.is_file() {
            continue;
        }

        let raw = fs::read(path)?;
        let slice = tail_bytes(&raw, BOOTSTRAP_LOG_BYTE_LIMIT as usize);
        let timestamp_unix_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        let events = String::from_utf8_lossy(slice)
            .lines()
            .filter(|line| !line.trim().is_empty())
            .rev()
            .take(BOOTSTRAP_LOG_LINE_LIMIT)
            .map(|line| DeviceLogEvent {
                device_id: device_id.to_string(),
                source: path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("syslog")
                    .to_string(),
                timestamp_unix_ms,
                level: None,
                message: line.to_string(),
                fields: Default::default(),
            })
            .collect::<Vec<_>>();
        if !events.is_empty() {
            return Ok(events.into_iter().rev().collect());
        }
    }

    bail!("no readable bootstrap log source found under /var/log/syslog or /var/log/messages")
}

fn tail_bytes(bytes: &[u8], max_len: usize) -> &[u8] {
    if bytes.len() <= max_len {
        bytes
    } else {
        &bytes[bytes.len() - max_len..]
    }
}
