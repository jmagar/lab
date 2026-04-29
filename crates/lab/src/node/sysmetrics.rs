use std::time::Duration;

use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, Networks, RefreshKind, System};

use crate::node::checkin::NodeStatus;

/// Collect live system metrics.
///
/// CPU usage requires two sysinfo refreshes with a brief sleep between them;
/// the first call always returns 0% per the sysinfo docs. Subsequent periodic
/// calls will return accurate deltas.
pub fn collect(node_id: &str) -> NodeStatus {
    let sys_refresh = RefreshKind::nothing()
        .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
        .with_memory(MemoryRefreshKind::everything());

    let mut sys = System::new_with_specifics(sys_refresh);
    sys.refresh_specifics(sys_refresh);
    std::thread::sleep(Duration::from_millis(250));
    sys.refresh_specifics(sys_refresh);

    let cpus = sys.cpus();
    let cpu_percent = if cpus.is_empty() {
        None
    } else {
        let avg = cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32;
        Some(avg)
    };
    let cores: Option<u64> = cpus.len().try_into().ok().filter(|&n: &u64| n > 0);
    let cpu_clock_mhz: Option<u64> = cpus.first().map(|c| c.frequency()).filter(|&f| f > 0);

    let memory_used_bytes = non_zero(sys.used_memory());
    let total_memory_bytes = non_zero(sys.total_memory());
    let uptime_seconds = Some(System::uptime());

    let (storage_used_bytes, total_storage_bytes) = root_disk_usage();
    let ips = local_ips();
    let cpu_temp_c = read_cpu_temp();

    NodeStatus {
        node_id: node_id.to_string(),
        connected: true,
        cpu_percent,
        memory_used_bytes,
        total_memory_bytes,
        storage_used_bytes,
        total_storage_bytes,
        os: Some(std::env::consts::OS.to_string()),
        ips,
        health: Some("healthy".to_string()),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
        uptime_seconds,
        cores,
        cpu_clock_mhz,
        cpu_temp_c,
        doctor_issues: vec![],
        active_claude_sessions: None,
        active_codex_sessions: None,
    }
}

fn non_zero(v: u64) -> Option<u64> {
    if v > 0 { Some(v) } else { None }
}

fn root_disk_usage() -> (Option<u64>, Option<u64>) {
    let mut disks = Disks::new_with_refreshed_list();
    disks.refresh(false);

    // Pick the disk whose mount point is "/" or closest parent of it.
    let best = disks
        .iter()
        .filter(|d| d.mount_point().to_str().is_some_and(|p| "/".starts_with(p)))
        .max_by_key(|d| d.mount_point().as_os_str().len());

    if let Some(disk) = best {
        let total = disk.total_space();
        let avail = disk.available_space();
        let used = total.saturating_sub(avail);
        return (non_zero(used), non_zero(total));
    }
    (None, None)
}

fn local_ips() -> Vec<String> {
    let mut networks = Networks::new_with_refreshed_list();
    networks.refresh(false);

    let mut ips = Vec::new();
    for (_name, data) in &networks {
        for addr in data.ip_networks() {
            let ip = addr.addr;
            // Skip loopback and link-local
            let s = ip.to_string();
            if s == "127.0.0.1" || s == "::1" || s.starts_with("169.254.") || s.starts_with("fe80:")
            {
                continue;
            }
            ips.push(s);
        }
    }
    ips.sort();
    ips.dedup();
    ips
}

/// Read the first CPU temperature from `/sys/class/thermal` on Linux.
fn read_cpu_temp() -> Option<f32> {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
            let mut zones: Vec<_> = entries
                .flatten()
                .filter(|e| {
                    e.file_name()
                        .to_str()
                        .is_some_and(|n| n.starts_with("thermal_zone"))
                })
                .collect();
            zones.sort_by_key(|e| e.file_name());
            for entry in zones {
                let path = entry.path();
                let zone_type = fs::read_to_string(path.join("type"))
                    .unwrap_or_default()
                    .trim()
                    .to_ascii_lowercase();
                // Only cpu/acpi/pkg zones; skip non-thermal sensors
                if !zone_type.is_empty()
                    && !["cpu", "x86", "acpi", "pkg", "tzone"]
                        .iter()
                        .any(|kw| zone_type.contains(kw))
                {
                    continue;
                }
                if let Ok(raw) = fs::read_to_string(path.join("temp")) {
                    if let Ok(millidegrees) = raw.trim().parse::<i64>() {
                        let celsius = millidegrees as f32 / 1000.0;
                        if celsius > 0.0 && celsius < 200.0 {
                            return Some(celsius);
                        }
                    }
                }
            }
        }
    }
    None
}
