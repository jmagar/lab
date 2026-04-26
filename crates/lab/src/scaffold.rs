//! Service onboarding scaffold orchestration.
//!
//! This is the top-level control point for generating a new service
//! module tree and the supporting repo patches.

mod patcher;
mod service;
mod templates;

pub use service::{
    FileOp, Result, ScaffoldConfig, ScaffoldError, ScaffoldKind, ScaffoldResult,
    validate_service_name,
};

use std::fs;
use std::io::Write as _;
use std::path::Path;

use tracing::info;

/// Generate the onboarding scaffold for one service.
pub fn scaffold_service(config: &ScaffoldConfig, dry_run: bool) -> Result<ScaffoldResult> {
    let service = validate_service_name(&config.service)?;

    let mut planned = service_file_ops(&service);
    planned.extend(patcher::compute_patches(&service, &config.repo_root)?);
    let mut created = Vec::new();
    let mut modified = Vec::new();

    if !dry_run {
        for op in &planned {
            let path = config.repo_root.join(&op.path);
            let existed = path.exists();
            let changed = write_file(&config.repo_root, op)?;
            if changed {
                if existed {
                    modified.push(op.path.clone());
                } else {
                    created.push(op.path.clone());
                }
            }
        }
    }

    Ok(ScaffoldResult {
        service,
        kind: config.kind,
        dry_run,
        planned,
        created,
        modified,
    })
}

fn service_file_ops(service: &str) -> Vec<FileOp> {
    vec![
        FileOp {
            path: Path::new("crates/lab-apis/src").join(format!("{service}.rs")),
            content: templates::lab_apis_service_template(service),
        },
        FileOp {
            path: Path::new("crates/lab-apis/src")
                .join(service)
                .join("client.rs"),
            content: templates::lab_apis_client_template(service),
        },
        FileOp {
            path: Path::new("crates/lab-apis/src")
                .join(service)
                .join("types.rs"),
            content: templates::lab_apis_types_template(service),
        },
        FileOp {
            path: Path::new("crates/lab-apis/src")
                .join(service)
                .join("error.rs"),
            content: templates::lab_apis_error_template(service),
        },
        FileOp {
            path: Path::new("crates/lab/src/dispatch").join(format!("{service}.rs")),
            content: templates::dispatch_entrypoint_template(service),
        },
        FileOp {
            path: Path::new("crates/lab/src/dispatch")
                .join(service)
                .join("catalog.rs"),
            content: templates::dispatch_catalog_template(service),
        },
        FileOp {
            path: Path::new("crates/lab/src/dispatch")
                .join(service)
                .join("client.rs"),
            content: templates::dispatch_client_template(service),
        },
        FileOp {
            path: Path::new("crates/lab/src/dispatch")
                .join(service)
                .join("dispatch.rs"),
            content: templates::dispatch_dispatch_template(service),
        },
        FileOp {
            path: Path::new("crates/lab/src/dispatch")
                .join(service)
                .join("params.rs"),
            content: templates::dispatch_params_template(service),
        },
        FileOp {
            path: Path::new("crates/lab/src/cli").join(format!("{service}.rs")),
            content: templates::adapter_cli_template(service),
        },
        FileOp {
            path: Path::new("crates/lab/src/api/services").join(format!("{service}.rs")),
            content: templates::adapter_api_template(service),
        },
        FileOp {
            path: Path::new("docs/coverage").join(format!("{service}.md")),
            content: templates::coverage_doc_template(service),
        },
    ]
}

/// Reject any path that contains a `..` (`ParentDir`) component.
///
/// Also checks that any existing parent directory in the path is not a symlink
/// that escapes the repository root. Canonicalization of the full path is not
/// used because the target file may not exist yet; instead we walk existing
/// ancestor directories and verify each resolved path stays within root.
///
/// All `FileOp` paths are constructed from hardcoded prefixes plus a validated
/// service name, so a `..` component can only appear via adversarial input.
fn validate_path_within_root(root: &Path, path: &Path) -> Result<()> {
    use std::path::Component;
    for component in path.components() {
        if component == Component::ParentDir {
            return Err(ScaffoldError::InvalidTarget {
                path: path.to_path_buf(),
                base: root.to_path_buf(),
            });
        }
    }

    // Walk existing ancestors to detect symlinks escaping the root.
    let abs = root.join(path);
    let mut check = abs.as_path();
    while let Some(parent) = check.parent() {
        if parent.exists() {
            if let Ok(canonical) = parent.canonicalize() {
                let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
                if !canonical.starts_with(&canonical_root) {
                    return Err(ScaffoldError::InvalidTarget {
                        path: path.to_path_buf(),
                        base: root.to_path_buf(),
                    });
                }
            }
            break;
        }
        check = parent;
    }

    Ok(())
}

fn write_file(repo_root: &Path, op: &FileOp) -> Result<bool> {
    validate_path_within_root(repo_root, &op.path)?;
    let path = repo_root.join(&op.path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| ScaffoldError::io(path.clone(), source))?;
    }

    if path.exists() {
        let existing =
            fs::read_to_string(&path).map_err(|source| ScaffoldError::io(path.clone(), source))?;
        if existing == op.content {
            return Ok(false);
        }
    }

    let mut tmp = tempfile::NamedTempFile::new_in(path.parent().unwrap_or(repo_root))
        .map_err(|source| ScaffoldError::io(path.clone(), source))?;
    std::io::Write::write_all(&mut tmp, op.content.as_bytes())
        .map_err(|source| ScaffoldError::io(path.clone(), source))?;
    tmp.flush()
        .map_err(|source| ScaffoldError::io(path.clone(), source))?;
    tmp.persist(&path)
        .map_err(|err| ScaffoldError::io(path.clone(), err.error))?;
    info!(path = %path.display(), "scaffold wrote file");
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn write_file_rejects_path_traversal() {
        let op = FileOp {
            path: std::path::PathBuf::from("../../../etc/evil"),
            content: "bad".to_string(),
        };
        let result = write_file(Path::new("/tmp"), &op);
        assert!(result.is_err(), "path traversal should be rejected");
    }
}
