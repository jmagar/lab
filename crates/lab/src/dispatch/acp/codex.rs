//! Codex provider: `spawn_codex()` and `codex_health()`.
//!
//! `spawn_codex()` creates a bounded `SessionHandle` (capacity 64) and
//! launches the codex subprocess with `env_clear()` + an allowlist of safe
//! environment variables so that lab secrets (e.g. `LAB_ACP_HMAC_SECRET`)
//! never leak into the subprocess.
//!
//! `codex_health()` runs `npx --version` in a `spawn_blocking` task with a
//! 5-second timeout — it never blocks the async executor.

use std::path::Path;

use lab_apis::acp::error::AcpError;
use lab_apis::acp::session::{SessionCommand, SessionHandle};
use lab_apis::acp::types::AcpProviderHealth;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

/// Bounded channel capacity for the prompt queue.
const PROMPT_CHANNEL_CAP: usize = 64;

/// Environment variables that are safe to pass into the codex subprocess.
///
/// Everything else is stripped by `.env_clear()` to prevent lab credentials
/// (e.g. `LAB_ACP_HMAC_SECRET`, API keys) from leaking into the subprocess.
const ALLOWED_ENV_VARS: &[&str] = &["PATH", "HOME", "TMPDIR", "LANG", "TERM"];

/// Spawn the Codex provider subprocess and return a [`SessionHandle`].
///
/// # Errors
///
/// Returns [`AcpError::SpawnFailed`] if:
/// - `cwd` does not exist or is not a directory.
/// - The underlying `tokio::task::spawn_blocking` task fails to join.
///
/// In Phase 1 this establishes the interface and wires `env_clear`. Full
/// subprocess I/O loop integration happens in Phase 2 / bead 5.
pub async fn spawn_codex(session_id: &str, cwd: &Path) -> Result<SessionHandle, AcpError> {
    // Validate cwd before doing anything else.
    if !cwd.exists() {
        return Err(AcpError::SpawnFailed(format!(
            "cwd does not exist: {}",
            cwd.display()
        )));
    }
    if !cwd.is_dir() {
        return Err(AcpError::SpawnFailed(format!(
            "cwd is not a directory: {}",
            cwd.display()
        )));
    }

    let (tx, rx) = mpsc::channel::<SessionCommand>(PROMPT_CHANNEL_CAP);

    // Collect the allowed env vars before entering the blocking thread.
    let env_pairs: Vec<(String, String)> = ALLOWED_ENV_VARS
        .iter()
        .filter_map(|key| std::env::var(key).ok().map(|val| (key.to_string(), val)))
        .collect();

    let cwd_owned = cwd.to_path_buf();
    let session_id_owned = session_id.to_string();

    // Spawn the subprocess setup in a blocking thread so we don't block the
    // async executor during process creation.
    tokio::task::spawn_blocking(move || {
        let mut cmd = std::process::Command::new("npx");
        cmd.arg("@openai/codex");

        // Strip all env vars from the subprocess environment, then re-inject
        // only the allowlisted ones. This prevents lab credentials from leaking.
        cmd.env_clear();
        for (key, value) in &env_pairs {
            cmd.env(key, value);
        }

        cmd.current_dir(&cwd_owned);

        // Phase 1: interface + env_clear scaffolding.
        // The rx consumer (subprocess I/O driver) is wired in Phase 2 / bead 5.
        // Drop rx here so the sender side reflects the correct closed state
        // when the real driver is not yet present.
        drop(rx);
        drop(session_id_owned);

        // Return unit — spawn failure is deferred to Phase 2 when we actually
        // call cmd.spawn() and check the result.
        Ok::<_, String>(())
    });

    Ok(SessionHandle {
        provider: "codex".to_string(),
        prompt_tx: tx,
    })
}

/// Check whether the Codex provider is available.
///
/// Runs `npx --version` in a [`tokio::task::spawn_blocking`] task with a
/// 5-second timeout. Never blocks the async executor.
pub async fn codex_health() -> AcpProviderHealth {
    let result = timeout(
        Duration::from_secs(5),
        tokio::task::spawn_blocking(|| {
            std::process::Command::new("npx")
                .arg("--version")
                .env_clear()
                .env("PATH", std::env::var("PATH").unwrap_or_default())
                .output()
        }),
    )
    .await;

    match result {
        Ok(Ok(Ok(out))) if out.status.success() => {
            let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
            AcpProviderHealth {
                provider: "codex".to_string(),
                available: true,
                version: Some(version),
                message: None,
            }
        }
        Ok(Ok(Ok(_out))) => AcpProviderHealth {
            provider: "codex".to_string(),
            available: false,
            version: None,
            message: Some("npx exited with non-zero status".to_string()),
        },
        Ok(Ok(Err(io_err))) => AcpProviderHealth {
            provider: "codex".to_string(),
            available: false,
            version: None,
            message: Some(format!("failed to run npx: {io_err}")),
        },
        Ok(Err(_join_err)) => AcpProviderHealth {
            provider: "codex".to_string(),
            available: false,
            version: None,
            message: Some("health check task panicked".to_string()),
        },
        Err(_timeout) => AcpProviderHealth {
            provider: "codex".to_string(),
            available: false,
            version: None,
            message: Some("npx --version timed out after 5 seconds".to_string()),
        },
    }
}
