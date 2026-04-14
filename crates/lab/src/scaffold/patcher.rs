//! Repo patch computation for service scaffolding.

mod source;
mod toml;

use std::fs;
use std::path::{Path, PathBuf};

use super::service::{FileOp, Result};

type PatchFn = fn(&str, &str) -> Result<String>;

pub fn compute_patches(name: &str, repo_root: &Path) -> Result<Vec<FileOp>> {
    let patches: &[(&str, PatchFn)] = &[
        ("crates/lab-apis/Cargo.toml", toml::patch_lab_apis_cargo),
        ("crates/lab/Cargo.toml", toml::patch_lab_cargo),
        ("crates/lab-apis/src/lib.rs", source::patch_lib_rs),
        ("crates/lab/src/dispatch.rs", source::patch_dispatch_rs),
        ("crates/lab/src/cli.rs", source::patch_cli_rs),
        (
            "crates/lab/src/mcp/services.rs",
            source::patch_mcp_services_rs,
        ),
        (
            "crates/lab/src/mcp/registry.rs",
            source::patch_mcp_registry_rs,
        ),
        (
            "crates/lab/src/api/services.rs",
            source::patch_api_services_rs,
        ),
        ("crates/lab/src/api/router.rs", source::patch_api_router_rs),
        (
            "crates/lab/src/dispatch/clients.rs",
            source::patch_dispatch_clients_rs,
        ),
        // Note: TUI service discovery is handled automatically via the MCP
        // registry (patch_mcp_registry_rs above). No separate tui/metadata.rs
        // patch is required — the TUI reads from build_default_registry().
    ];

    let mut ops = Vec::new();
    for (path_str, patch) in patches {
        collect_patch(name, repo_root, path_str, patch, &mut ops)?;
    }
    Ok(ops)
}

fn collect_patch<F>(
    name: &str,
    repo_root: &Path,
    rel_path: &str,
    patch: F,
    ops: &mut Vec<FileOp>,
) -> Result<()>
where
    F: Fn(&str, &str) -> Result<String>,
{
    let relative = PathBuf::from(rel_path);
    let absolute = repo_root.join(&relative);
    let raw = fs::read_to_string(&absolute)
        .map_err(|source| super::service::ScaffoldError::io(absolute.clone(), source))?;
    // Normalize CRLF → LF so patch anchors match on Windows checkouts.
    let content = raw.replace("\r\n", "\n");
    let patched = patch(name, &content)?;
    if patched != content {
        ops.push(FileOp {
            path: relative,
            content: patched,
        });
    }
    Ok(())
}
