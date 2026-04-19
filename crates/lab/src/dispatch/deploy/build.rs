//! Local release build + sha256 hashing + disk preflight.

use lab_apis::deploy::DeployError;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};

/// Artifact produced by a successful local build.
#[derive(Debug, Clone)]
pub struct BuildOutcome {
    pub path: PathBuf,
    pub sha256: String,
    pub size_bytes: u64,
    pub target_triple: String,
}

/// Run `cargo build --release --all-features -p lab` and hash the output.
pub async fn build_release() -> Result<BuildOutcome, DeployError> {
    let free = tokio::task::spawn_blocking(estimate_free_bytes)
        .await
        .map_err(|e| DeployError::BuildFailed {
            reason: format!("disk-space check join: {e}"),
        })??;
    check_disk_space(free, 1_500_000_000)?;
    let output = tokio::process::Command::new("cargo")
        .args(["build", "--release", "--all-features", "-p", "lab"])
        .output()
        .await
        .map_err(|e| DeployError::BuildFailed {
            reason: format!("spawn cargo: {e}"),
        })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let tail: Vec<&str> = stderr.lines().rev().take(10).collect();
        let tail = tail.into_iter().rev().collect::<Vec<_>>().join("\n");
        return Err(DeployError::BuildFailed { reason: tail });
    }
    let path = expected_artifact_path("lab");
    // Stat + sha256 + host-triple detection are all blocking I/O or subprocess
    // calls; run them inside spawn_blocking to avoid stalling the async runtime.
    let (metadata, sha256, target_triple) = tokio::task::spawn_blocking({
        let p = path.clone();
        move || -> Result<_, DeployError> {
            let meta = std::fs::metadata(&p).map_err(|e| DeployError::BuildFailed {
                reason: format!("stat artifact: {e}"),
            })?;
            let sha256 = sha256_file_blocking(&p)?;
            Ok((meta, sha256, detect_host_triple()))
        }
    })
    .await
    .map_err(|e| DeployError::BuildFailed {
        reason: format!("post-build join: {e}"),
    })??;
    Ok(BuildOutcome {
        path,
        sha256,
        size_bytes: metadata.len(),
        target_triple,
    })
}

/// Path where cargo places the binary for `target_triple`.
///
/// - **Host triple** (`target_triple == detect_host_triple()`): `target/release/<bin>`
/// - **Cross-compilation target**: `target/<triple>/release/<bin>`
/// - When `target_triple` contains `"windows"`, `.exe` is appended even on a
///   non-Windows build host. Never use `cfg!(target_os)` here — this is about
///   the *target*, not the host.
pub fn expected_artifact_path_for(bin: &str, target_triple: &str) -> PathBuf {
    let name = if target_triple.contains("windows") {
        format!("{bin}.exe")
    } else {
        bin.to_string()
    };
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("target"))
        .unwrap_or_else(|| PathBuf::from("target"));
    // Cargo places the artifact under `target/<triple>/release/` only when
    // cross-compiling (i.e., the target differs from the host triple).
    if target_triple == detect_host_triple() {
        workspace.join("release").join(&name)
    } else {
        workspace.join(target_triple).join("release").join(&name)
    }
}

/// Path under the workspace `target/release/` directory (host triple).
///
/// Delegates to `expected_artifact_path_for` using the current host triple.
/// Use `expected_artifact_path_for` directly when cross-compiling.
pub fn expected_artifact_path(bin: &str) -> PathBuf {
    expected_artifact_path_for(bin, &detect_host_triple())
}

/// Blocking SHA-256 of a file; call from `spawn_blocking`.
pub fn sha256_file_blocking(path: &Path) -> Result<String, DeployError> {
    let mut f = std::fs::File::open(path).map_err(|e| DeployError::BuildFailed {
        reason: format!("open: {e}"),
    })?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = f.read(&mut buf).map_err(|e| DeployError::BuildFailed {
            reason: format!("read: {e}"),
        })?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// Error out when the local filesystem has less than `required` bytes free.
pub fn check_disk_space(available: u64, required: u64) -> Result<(), DeployError> {
    if available < required {
        return Err(DeployError::PreflightFailed {
            host: "localhost".into(),
            reason: format!("insufficient disk space: have {available} need {required}"),
        });
    }
    Ok(())
}

fn estimate_free_bytes() -> Result<u64, DeployError> {
    // Use POSIX-compatible `df -k` (kilobytes) — works on Linux, BSD, and macOS.
    // Output format: Filesystem 1K-blocks Used Available Capacity Mounted-on
    // "Available" is column index 3 (0-based), in 1 KiB units.
    let out = std::process::Command::new("df")
        .arg("-k")
        .arg(".")
        .output();
    if let Ok(o) = out {
        if o.status.success() {
            if let Some(line) = String::from_utf8_lossy(&o.stdout).lines().nth(1) {
                let mut fields = line.split_whitespace();
                // Skip: Filesystem, 1K-blocks, Used; take Available
                if let Some(avail_kib) = fields.nth(3) {
                    if let Ok(kib) = avail_kib.parse::<u64>() {
                        return Ok(kib.saturating_mul(1024));
                    }
                }
            }
        }
    }
    // df unavailable or unparseable — skip the disk check rather than blocking.
    tracing::warn!("could not determine free disk space; skipping preflight disk check");
    Ok(u64::MAX)
}

fn detect_host_triple() -> String {
    let out = std::process::Command::new("rustc").arg("-vV").output();
    if let Ok(o) = out {
        if o.status.success() {
            for line in String::from_utf8_lossy(&o.stdout).lines() {
                if let Some(rest) = line.strip_prefix("host: ") {
                    return rest.trim().to_string();
                }
            }
        }
    }
    std::env::consts::ARCH.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn sha256_of_known_bytes_is_deterministic() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("artifact");
        std::fs::write(&path, b"lab-binary-v1").unwrap();
        let hex = sha256_file_blocking(&path).unwrap();
        assert_eq!(hex.len(), 64);
        assert_eq!(hex, sha256_file_blocking(&path).unwrap());
    }

    #[test]
    fn build_target_path_matches_cargo_layout() {
        // Host triple → target/release/<bin> (no triple in path).
        let host = detect_host_triple();
        let p = expected_artifact_path_for("lab", &host);
        assert!(p.ends_with("target/release/lab"), "got {}", p.display());
    }

    #[test]
    fn cross_target_path_includes_triple() {
        // A cross-compilation target that differs from the host must include
        // the triple so cargo's output directory layout is matched correctly.
        let host = detect_host_triple();
        let cross = if host.contains("x86_64") {
            "aarch64-unknown-linux-gnu"
        } else {
            "x86_64-unknown-linux-gnu"
        };
        let p = expected_artifact_path_for("lab", cross);
        let expected = format!("target/{cross}/release/lab");
        assert!(p.ends_with(&expected), "got {}", p.display());
    }

    #[test]
    fn windows_target_appends_exe_suffix() {
        let p = expected_artifact_path_for("lab", "x86_64-pc-windows-msvc");
        assert!(
            p.ends_with("target/release/lab.exe"),
            "got {}",
            p.display()
        );
    }

    #[test]
    fn disk_preflight_rejects_below_threshold() {
        let err = check_disk_space(10, 100).unwrap_err();
        assert_eq!(err.kind(), "preflight_failed");
    }
}
