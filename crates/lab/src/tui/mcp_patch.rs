//! .mcp.json atomic patcher — implementation in lab-iuk.3.
use std::path::Path;

/// Patch the `--services` array in `.mcp.json`, enabling or disabling a service.
/// Full implementation in lab-iuk.3.
pub fn patch_mcp_json(_path: &Path, _service_name: &str, _enabled: bool) -> anyhow::Result<()> {
    anyhow::bail!("mcp_patch not yet implemented")
}
