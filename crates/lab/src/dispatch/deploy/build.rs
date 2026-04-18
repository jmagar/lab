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
    check_disk_space(estimate_free_bytes()?, 1_500_000_000)?;
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
    let metadata = std::fs::metadata(&path).map_err(|e| DeployError::BuildFailed {
        reason: format!("stat artifact: {e}"),
    })?;
    let sha256 = tokio::task::spawn_blocking({
        let p = path.clone();
        move || sha256_file_blocking(&p)
    })
    .await
    .map_err(|e| DeployError::BuildFailed {
        reason: format!("sha256 join: {e}"),
    })??;
    Ok(BuildOutcome {
        path,
        sha256,
        size_bytes: metadata.len(),
        target_triple: detect_host_triple(),
    })
}

/// Path under the workspace `target/release/` directory.
pub fn expected_artifact_path(bin: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("target").join("release").join(bin))
        .unwrap_or_else(|| PathBuf::from("target").join("release").join(bin))
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
    let out = std::process::Command::new("df")
        .arg("--output=avail")
        .arg("-B1")
        .arg(".")
        .output();
    if let Ok(o) = out {
        if o.status.success() {
            if let Some(line) = String::from_utf8_lossy(&o.stdout).lines().nth(1) {
                if let Ok(n) = line.trim().parse::<u64>() {
                    return Ok(n);
                }
            }
        }
    }
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
        let p = expected_artifact_path("lab");
        assert!(p.ends_with("target/release/lab"), "got {}", p.display());
    }

    #[test]
    fn disk_preflight_rejects_below_threshold() {
        let err = check_disk_space(10, 100).unwrap_err();
        assert_eq!(err.kind(), "preflight_failed");
    }
}
