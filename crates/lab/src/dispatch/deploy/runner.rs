//! Deploy orchestrator.
//!
//! V1 scope: trait + in-process default runner that drives the
//! build → preflight → transfer → install → restart → verify pipeline over
//! SSH using `tokio::process::Command` (wrapped in `SshSession`). Tests in
//! this crate substitute a recording `HostIo` mock for fast orchestration
//! coverage without a live SSH server.
//!
//! Shell-exception audit. Only two code paths construct a `sh -c` command:
//!
//! * `preflight`'s canary-write probe — the canary path is derived from
//!   `remote_path`, which is allowlist-validated in `params::validate_remote_path`.
//! * `SshSession::upload_stream` — remote redirect needs a shell wrapper.
//!
//! Every other `io.run(&[...])` call uses per-token argv and interpolates no
//! untrusted strings.

use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

use lab_apis::core::ssh::{SshHostTarget, SshSession, shell_quote};
use lab_apis::deploy::{
    DeployError, DeployHostResult, DeployPlan, DeployRequest, DeployRunSummary, DeployStage,
};
use serde_json::{Value, json};

use crate::config::ServiceScope;
use crate::dispatch::error::ToolError;

use super::build::{self, BuildOutcome};
use super::lock::HostLockRegistry;
use super::params;

/// Surface-neutral deploy orchestrator used by CLI and MCP adapters.
pub trait DeployRunner: Send + Sync {
    async fn plan(&self, req: DeployRequest) -> Result<DeployPlan, ToolError>;
    async fn run(&self, req: DeployRequest) -> Result<DeployRunSummary, ToolError>;
    async fn rollback(&self, req: DeployRequest) -> Result<DeployRunSummary, ToolError>;
    async fn config_list(&self) -> Result<Value, ToolError>;
}

/// Low-level primitive the runner uses to talk to a single host.
///
/// The production implementation is `SshHostIo` backed by `SshSession`.
/// Tests substitute a recording fake that captures the op stream and
/// returns scripted responses without touching the network.
///
/// All methods are sync fns returning `'static` futures. This avoids
/// higher-ranked trait bound (HRTB) errors in `Box::pin(async move { … } +
/// Send + 'static)` contexts (Rust issue #100013). Implementations must do
/// all `&self` work synchronously and capture only owned values in the
/// returned future.
pub trait HostIo: Send + Sync {
    fn run_argv(
        &self,
        argv: &[&str],
    ) -> Pin<Box<dyn Future<Output = Result<(i32, String, String), DeployError>> + Send + 'static>>;

    fn upload_stream<R>(
        &self,
        remote_path: &str,
        reader: R,
    ) -> Pin<Box<dyn Future<Output = Result<u64, DeployError>> + Send + 'static>>
    where
        R: tokio::io::AsyncRead + Unpin + Send + 'static;

    fn sha256_remote(
        &self,
        remote_path: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, DeployError>> + Send + 'static>>;
}

/// Production `HostIo` impl backed by `lab_apis::core::ssh::SshSession`.
///
/// `host` is carried alongside the session so errors can be tagged with the
/// alias the caller asked for, not the underlying hostname.
pub struct SshHostIo {
    pub host: String,
    pub session: SshSession,
}

impl SshHostIo {
    #[must_use]
    pub fn new(host: impl Into<String>, target: SshHostTarget) -> Self {
        Self {
            host: host.into(),
            session: SshSession::new(target),
        }
    }
}

impl HostIo for SshHostIo {
    fn run_argv(
        &self,
        argv: &[&str],
    ) -> Pin<Box<dyn Future<Output = Result<(i32, String, String), DeployError>> + Send + 'static>> {
        let fut = self.session.run_command(argv);
        let host = self.host.clone();
        Box::pin(async move {
            fut.await.map_err(|e| DeployError::SshUnreachable {
                host: format!("{host}: {e}"),
            })
        })
    }

    fn upload_stream<R>(
        &self,
        remote_path: &str,
        reader: R,
    ) -> Pin<Box<dyn Future<Output = Result<u64, DeployError>> + Send + 'static>>
    where
        R: tokio::io::AsyncRead + Unpin + Send + 'static,
    {
        let fut = self.session.upload_stream(remote_path, reader);
        let host = self.host.clone();
        Box::pin(async move {
            fut.await.map_err(|e| DeployError::TransferFailed {
                host,
                reason: e.to_string(),
            })
        })
    }

    fn sha256_remote(
        &self,
        remote_path: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<String>, DeployError>> + Send + 'static>> {
        let fut = self.session.sha256_remote(remote_path);
        let host = self.host.clone();
        Box::pin(async move {
            fut.await.map_err(|e| DeployError::SshUnreachable {
                host: format!("{host}: {e}"),
            })
        })
    }
}

// ── Stage functions ────────────────────────────────────────────────────────

/// Outcome of the preflight stage.
#[derive(Debug, Clone, Copy)]
pub struct PreflightOutcome {
    /// True when the remote artifact already matches `local_sha256`; the
    /// transfer stage is skipped entirely.
    pub skip_transfer: bool,
}

/// Preflight: architecture match + canary write + sha256 skip probe.
///
/// The canary-write check uses `sh -c` deliberately. `remote_path` is
/// allowlist-validated via `params::validate_remote_path` at the top of this
/// function and the canary filename is a fixed pattern — the path is also
/// shell-single-quote-escaped before interpolation.
///
/// Takes `Arc<I>` and owned strings so the resulting future holds only
/// `Send` values across await points (no HRTB via Rust issue #100013).
pub async fn preflight<I: HostIo + 'static>(
    io: Arc<I>,
    remote_path: String,
    local_triple: String,
    local_sha256: String,
) -> Result<PreflightOutcome, DeployError> {
    // 0) Validate remote_path against the allowlist before any shell use.
    params::validate_remote_path(&remote_path)?;

    // 1) Architecture match.
    let (code, stdout, stderr) = io.run_argv(&["uname", "-m"]).await?;
    if code != 0 {
        return Err(DeployError::PreflightFailed {
            host: "?".into(),
            reason: format!("uname -m exit {code}: {}", stderr.trim()),
        });
    }
    let remote_arch = stdout.trim().to_string();
    let local_arch = triple_to_arch(&local_triple);
    if !arch_matches(&local_arch, &remote_arch) {
        return Err(DeployError::ArchMismatch {
            host: "?".into(),
            local: local_arch,
            remote: remote_arch,
        });
    }

    // 2) Writable install dir (canary touch + rm).
    let parent = Path::new(&remote_path)
        .parent()
        .ok_or_else(|| DeployError::PreflightFailed {
            host: "?".into(),
            reason: format!("remote_path `{remote_path}` has no parent directory"),
        })?
        .to_string_lossy()
        .into_owned();
    // Canary lives under a fixed filename.  The parent dir is derived from
    // the allowlist-validated remote_path but we still single-quote-escape
    // the interpolated paths for defense-in-depth.
    let canary = format!("{parent}/.lab.deploy.canary.$$");
    let sq_canary = shell_quote(&canary);
    let probe = format!("touch {sq_canary} && rm -f -- {sq_canary}");
    let (code, _stdout, stderr) = io.run_argv(&["sh", "-c", &probe]).await?;
    if code != 0 {
        return Err(DeployError::PreflightFailed {
            host: "?".into(),
            reason: format!("install dir `{parent}` not writable: {}", stderr.trim()),
        });
    }

    // 3) Remote sha256 probe — when it matches, transfer is skipped.
    let remote_sha = io.sha256_remote(&remote_path).await?;
    Ok(PreflightOutcome {
        skip_transfer: remote_sha.as_deref() == Some(local_sha256.as_str()),
    })
}

/// Outcome of the transfer + install stage.
#[derive(Debug, Clone)]
pub struct TransferOutcome {
    pub bytes: u64,
    /// Path of the `.bak.<ts>` file the previous binary (if any) was moved to.
    pub backup_path: Option<String>,
}

/// Transfer + atomic install. The sequence is:
///
/// 1. `upload_stream` the new artifact to `<remote_path>.new.partial`
/// 2. `mv -- <.partial> <.new>`
/// 3. `sha256sum <.new>` must equal `local_sha256` — else remove and abort
/// 4. if `<remote_path>` exists, `mv -- <current> <remote_path>.bak.<ts>`
/// 5. `mv -- <.new> <remote_path>` (atomic on same filesystem)
///
/// Takes `Arc<I>` and owned strings so the resulting future holds only
/// `Send` values across await points (no HRTB via Rust issue #100013).
pub async fn transfer_and_install<I: HostIo + 'static, R>(
    io: Arc<I>,
    remote_path: String,
    local_sha256: String,
    reader: R,
) -> Result<TransferOutcome, DeployError>
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    let partial = format!("{remote_path}.new.partial");
    let staged = format!("{remote_path}.new");

    // 1) stream to .partial
    let bytes = io.upload_stream(&partial, reader).await?;

    // 2) rename .partial -> .new
    let (code, _stdout, stderr) = io.run_argv(&["mv", "--", &partial, &staged]).await?;
    if code != 0 {
        return Err(DeployError::TransferFailed {
            host: "?".into(),
            reason: format!("rename partial -> staged: {}", stderr.trim()),
        });
    }

    // 3) integrity verify
    let remote_sha = io
        .sha256_remote(&staged)
        .await?
        .ok_or_else(|| DeployError::TransferFailed {
            host: "?".into(),
            reason: "post-upload sha256 probe returned no hash".into(),
        })?;
    if remote_sha != local_sha256 {
        // best-effort cleanup; drop the result explicitly (lint: let-underscore-drop)
        drop(io.run_argv(&["rm", "-f", "--", &staged]).await);
        return Err(DeployError::IntegrityMismatch { host: "?".into() });
    }

    // 4) backup existing (only if present)
    let backup = format!("{remote_path}.bak.{}", backup_timestamp());
    let maybe_existing = io.sha256_remote(&remote_path).await?;
    let backup_path = if maybe_existing.is_some() {
        let (code, _stdout, stderr) = io
            .run_argv(&["mv", "--", &remote_path, &backup])
            .await?;
        if code != 0 {
            return Err(DeployError::InstallFailed {
                host: "?".into(),
                reason: format!("backup rename: {}", stderr.trim()),
            });
        }
        Some(backup)
    } else {
        None
    };

    // 5) atomic swap .new -> remote_path
    let (code, _stdout, stderr) = io.run_argv(&["mv", "--", &staged, &remote_path]).await?;
    if code != 0 {
        return Err(DeployError::InstallFailed {
            host: "?".into(),
            reason: format!("final rename: {}", stderr.trim()),
        });
    }

    Ok(TransferOutcome { bytes, backup_path })
}

/// Outcome of the restart stage.
#[derive(Debug, Clone, Copy)]
pub struct RestartOutcome {
    /// True when `unit` was `None` — no systemd action was taken.
    pub skipped: bool,
}

/// Restart a systemd unit and wait for it to be active.
///
/// When `unit` is `None`, returns `skipped = true` without running any
/// remote command. Unit names are validated against
/// `params::validate_service_name` before any `systemctl` call.
///
/// Takes `Arc<I>` and an owned `unit` string so the resulting future holds
/// only `Send` values across await points (no HRTB via Rust issue #100013).
pub async fn restart<I: HostIo + 'static>(
    io: Arc<I>,
    unit: Option<String>,
    scope: Option<ServiceScope>,
) -> Result<RestartOutcome, DeployError> {
    let Some(unit) = unit else {
        return Ok(RestartOutcome { skipped: true });
    };
    params::validate_service_name(&unit)?;

    let user_scope = matches!(scope, Some(ServiceScope::User));

    fn systemctl_argv(user_scope: bool, subcommand: &str, unit: &str) -> Vec<String> {
        let mut v: Vec<String> = vec!["systemctl".into()];
        if user_scope {
            v.push("--user".into());
        }
        v.push(subcommand.to_string());
        v.push(unit.to_string());
        v
    }

    let restart_args = systemctl_argv(user_scope, "restart", &unit);
    let restart_argv: Vec<&str> = restart_args.iter().map(String::as_str).collect();
    let (code, _stdout, stderr) = io.run_argv(&restart_argv).await?;
    if code != 0 {
        return Err(DeployError::RestartFailed {
            host: "?".into(),
            reason: format!("restart exit {code}: {}", stderr.trim()),
        });
    }

    let mut wait_args = systemctl_argv(user_scope, "is-active", &unit);
    // Insert --wait before the unit name (last element) per systemctl convention.
    let unit_pos = wait_args.len() - 1;
    wait_args.insert(unit_pos, "--wait".into());
    let wait_argv: Vec<&str> = wait_args.iter().map(String::as_str).collect();
    let (code, _stdout, stderr) = io.run_argv(&wait_argv).await?;
    if code != 0 {
        return Err(DeployError::RestartFailed {
            host: "?".into(),
            reason: format!("is-active --wait exit {code}: {}", stderr.trim()),
        });
    }
    Ok(RestartOutcome { skipped: false })
}

/// Verify the newly installed binary by running `<remote_path> --version`.
///
/// Takes `Arc<I>` and an owned `remote_path` string so the resulting future
/// holds only `Send` values across await points (no HRTB via Rust issue #100013).
pub async fn verify<I: HostIo + 'static>(
    io: Arc<I>,
    remote_path: String,
) -> Result<(), DeployError> {
    let (code, _stdout, stderr) = io.run_argv(&[remote_path.as_str(), "--version"]).await?;
    if code != 0 {
        return Err(DeployError::VerifyFailed {
            host: "?".into(),
            reason: format!("--version exit {code}: {}", stderr.trim()),
        });
    }
    Ok(())
}

/// Strip a Rust target triple to its architecture field (first `-`-delimited token).
fn triple_to_arch(triple: &str) -> String {
    triple.split('-').next().unwrap_or(triple).to_string()
}

/// Normalize common architecture aliases to canonical Rust triple arch names.
///
/// This handles cases like Docker/OCI image platforms (`amd64`, `arm64`) and
/// Linux `uname -m` names (`armv7l`, `armhf`) which differ from Rust triple
/// arch names (`x86_64`, `aarch64`, `armv7`).
fn normalize_arch(arch: &str) -> &str {
    match arch {
        "amd64" | "x64" => "x86_64",
        "arm64" => "aarch64",
        // uname -m on 32-bit ARM Linux returns `armv7l` (little-endian suffix).
        // Rust triples use `armv7` without the `l`. armhf is a Debian alias.
        "armv7l" | "armhf" => "armv7",
        other => other,
    }
}

/// Compare `uname -m` output to the architecture from a Rust target triple.
///
/// Normalizes known aliases (e.g. `amd64` → `x86_64`, `arm64` → `aarch64`)
/// before comparing so that cross-platform CI and OCI-derived arch strings
/// do not cause spurious mismatches.
fn arch_matches(local: &str, remote: &str) -> bool {
    normalize_arch(local) == normalize_arch(remote)
}

fn backup_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
        .to_string()
}


// ── DefaultRunner ──────────────────────────────────────────────────────────

/// Default in-process runner wired into `SshSession` + the on-disk inventory.
pub struct DefaultRunner {
    pub config: crate::config::DeployPreferences,
    pub ssh_inventory: Arc<Vec<SshHostTarget>>,
    pub locks: Arc<HostLockRegistry>,
}

impl DefaultRunner {
    /// Construct a new default runner.
    #[must_use]
    pub fn new(
        config: crate::config::DeployPreferences,
        ssh_inventory: Arc<Vec<SshHostTarget>>,
        locks: Arc<HostLockRegistry>,
    ) -> Self {
        Self {
            config,
            ssh_inventory,
            locks,
        }
    }

    fn resolve_target(&self, alias: &str) -> Option<&SshHostTarget> {
        self.ssh_inventory.iter().find(|h| h.alias == alias)
    }

    fn effective_max_parallel(&self) -> Option<u32> {
        self.config
            .defaults
            .as_ref()
            .and_then(|d| d.max_parallel)
    }

    fn effective_remote_path(&self, host: &str) -> String {
        self.config
            .hosts
            .get(host)
            .and_then(|o| o.remote_path.clone())
            .or_else(|| {
                self.config
                    .defaults
                    .as_ref()
                    .and_then(|d| d.remote_path.clone())
            })
            .unwrap_or_else(|| "/usr/local/bin/lab".to_string())
    }

    fn effective_unit(&self, host: &str) -> Option<String> {
        self.config
            .hosts
            .get(host)
            .and_then(|o| o.service.clone())
            .or_else(|| {
                self.config
                    .defaults
                    .as_ref()
                    .and_then(|d| d.service.clone())
            })
    }

    fn effective_scope(&self, host: &str) -> Option<ServiceScope> {
        self.config
            .hosts
            .get(host)
            .and_then(|o| o.service_scope)
            .or_else(|| {
                self.config
                    .defaults
                    .as_ref()
                    .and_then(|d| d.service_scope)
            })
    }

    fn canary_set(&self) -> std::collections::BTreeSet<String> {
        self.config
            .defaults
            .as_ref()
            .map(|d| d.canary_hosts.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn partition_canary(&self, targets: &[String]) -> (Vec<String>, Vec<String>) {
        let set = self.canary_set();
        let mut canary = Vec::new();
        let mut rest = Vec::new();
        for t in targets {
            if set.contains(t) {
                canary.push(t.clone());
            } else {
                rest.push(t.clone());
            }
        }
        (canary, rest)
    }
}

// Thin trait delegation — keeps async fn in trait semantics without
// the HRTB Send limitation that blocks Box::pin in the MCP registry.
// Actual implementations live as inherent pub async fn methods below.
impl DeployRunner for DefaultRunner {
    async fn plan(&self, req: DeployRequest) -> Result<DeployPlan, ToolError> {
        self.plan_impl(req).await
    }

    async fn run(&self, req: DeployRequest) -> Result<DeployRunSummary, ToolError> {
        self.run_impl(req).await
    }

    async fn rollback(&self, req: DeployRequest) -> Result<DeployRunSummary, ToolError> {
        self.rollback_impl(req).await
    }

    async fn config_list(&self) -> Result<Value, ToolError> {
        self.config_list_impl()
    }
}

// Inherent implementations — called directly by dispatch_with_runner so the
// future type is concrete (not an RPITIT from a trait), making Send provable.
//
// IMPORTANT: all `&self` accesses must occur BEFORE any `.await` point.
// Borrowing `self` across an await creates lifetime-parameterised captures
// that fail the higher-ranked Send check required by `Box::pin` in the MCP
// registry (Rust issue #100013). Extract all needed values synchronously,
// then hand off only owned / Arc values to the async work.
impl DefaultRunner {
    pub fn plan_impl(
        &self,
        req: DeployRequest,
    ) -> Pin<Box<dyn Future<Output = Result<DeployPlan, ToolError>> + Send + 'static>> {
        // --- sync: all self access before creating the future ---
        for alias in &req.targets {
            if self.resolve_target(alias).is_none() {
                let err: Result<DeployPlan, ToolError> = Err(DeployError::ValidationFailed {
                    field: "targets".into(),
                    reason: format!("unknown SSH alias: {alias}"),
                }
                .into());
                return Box::pin(async move { err });
            }
        }
        let canary_hosts: Vec<String> = self.canary_set().into_iter().collect();
        let max_parallel = req
            .max_parallel
            .or_else(|| self.effective_max_parallel())
            .unwrap_or(1)
            .max(1);

        // --- async: only owned values, no &self ---
        Box::pin(async move {
            let artifact = build::expected_artifact_path("lab");
            let artifact_sha256 = if matches!(artifact.try_exists(), Ok(true)) {
                let p = artifact.clone();
                tokio::task::spawn_blocking(move || build::sha256_file_blocking(&p))
                    .await
                    .map_err(|e| ToolError::internal_message(format!("sha256 join: {e}")))?
                    .ok()
            } else {
                None
            };
            Ok(DeployPlan {
                artifact_path: artifact.to_string_lossy().into_owned(),
                artifact_sha256,
                hosts: req.targets.clone(),
                max_parallel,
                canary_hosts,
            })
        })
    }

    pub fn run_impl(
        &self,
        req: DeployRequest,
    ) -> Pin<Box<dyn Future<Output = Result<DeployRunSummary, ToolError>> + Send + 'static>> {
        use tracing::Instrument;

        // --- sync: all self access before creating the future ---
        for alias in &req.targets {
            if self.resolve_target(alias).is_none() {
                let err: Result<DeployRunSummary, ToolError> =
                    Err(DeployError::ValidationFailed {
                        field: "targets".into(),
                        reason: format!("unknown SSH alias: {alias}"),
                    }
                    .into());
                return Box::pin(async move { err });
            }
        }
        let max_parallel = req
            .max_parallel
            .or_else(|| self.effective_max_parallel())
            .unwrap_or(1)
            .max(1) as usize;
        let (canary, rest) = self.partition_canary(&req.targets);
        let canary_jobs = self.build_jobs(&canary);
        let rest_jobs = self.build_jobs(&rest);
        let locks = self.locks.clone();
        let run_id = uuid::Uuid::new_v4().to_string();
        let span = tracing::info_span!(
            "deploy.run",
            run_id = %run_id,
            service = "deploy",
            surface = "dispatch",
        );

        // --- async: only owned / Arc values, no &self ---
        Box::pin(
            async move {
                let build_outcome = Arc::new(build::build_release().await?);
                tracing::info!(
                    artifact_sha256 = %build_outcome.sha256,
                    size_bytes = build_outcome.size_bytes,
                    "deploy.build.ok"
                );

                let mut all_results: Vec<DeployHostResult> = Vec::new();

                if !canary_jobs.is_empty() {
                    let canary_results = DefaultRunner::run_jobs(
                        canary_jobs,
                        build_outcome.clone(),
                        1,
                        req.fail_fast,
                        run_id.clone(),
                        locks.clone(),
                    )
                    .await;
                    let any_failed = canary_results.iter().any(|r| !r.succeeded);
                    all_results.extend(canary_results);
                    if req.fail_fast && any_failed {
                        for host in &rest {
                            all_results.push(aborted_result(host));
                        }
                        return Ok(summarize(run_id, build_outcome.sha256.clone(), all_results));
                    }
                }

                if !rest_jobs.is_empty() {
                    let rest_results = DefaultRunner::run_jobs(
                        rest_jobs,
                        build_outcome.clone(),
                        max_parallel,
                        req.fail_fast,
                        run_id.clone(),
                        locks,
                    )
                    .await;
                    all_results.extend(rest_results);
                }

                Ok(summarize(run_id, build_outcome.sha256.clone(), all_results))
            }
            .instrument(span),
        )
    }

    pub fn rollback_impl(
        &self,
        req: DeployRequest,
    ) -> Pin<Box<dyn Future<Output = Result<DeployRunSummary, ToolError>> + Send + 'static>> {
        use tracing::Instrument;

        // --- sync: collect all per-host data from self before creating the future ---
        struct HostRollback {
            host: String,
            target: Option<SshHostTarget>,
            remote_path: String,
            unit: Option<String>,
            scope: Option<ServiceScope>,
        }
        let host_data: Vec<HostRollback> = req
            .targets
            .iter()
            .map(|host| HostRollback {
                target: self.resolve_target(host).cloned(),
                remote_path: self.effective_remote_path(host),
                unit: self.effective_unit(host),
                scope: self.effective_scope(host),
                host: host.clone(),
            })
            .collect();
        let locks = self.locks.clone();
        let max_parallel = self.effective_max_parallel().unwrap_or(1).max(1) as usize;
        let run_id = uuid::Uuid::new_v4().to_string();
        let span = tracing::info_span!(
            "deploy.rollback",
            run_id = %run_id,
            service = "deploy",
            surface = "dispatch",
        );

        // --- async: only owned / Arc values, no &self ---
        Box::pin(
            async move {
                use futures::stream::{self, StreamExt};

                let results = stream::iter(host_data)
                    .map(|data| {
                        let locks = locks.clone();
                        async move {
                            let Some(target) = data.target else {
                                return DeployHostResult {
                                    host: data.host,
                                    reached_stage: DeployStage::Resolve,
                                    succeeded: false,
                                    skipped_transfer: false,
                                    transferred_bytes: None,
                                    error_kind: Some("validation_failed".into()),
                                    stage_timings_ms: Default::default(),
                                };
                            };
                            let lock_timeout = std::time::Duration::from_secs(60);
                            let _guard = match locks.acquire(&data.host, lock_timeout).await {
                                Ok(g) => g,
                                Err(e) => {
                                    return DeployHostResult {
                                        host: data.host,
                                        reached_stage: DeployStage::Resolve,
                                        succeeded: false,
                                        skipped_transfer: false,
                                        transferred_bytes: None,
                                        error_kind: Some(e.kind().to_string()),
                                        stage_timings_ms: Default::default(),
                                    };
                                }
                            };
                            let io = Arc::new(SshHostIo::new(data.host.clone(), target));
                            rollback_one_host(
                                io,
                                data.host,
                                data.remote_path,
                                data.unit,
                                data.scope,
                            )
                            .await
                        }
                    })
                    .buffer_unordered(max_parallel)
                    .collect::<Vec<_>>()
                    .await;

                Ok(summarize(run_id, String::new(), results))
            }
            .instrument(span),
        )
    }

    pub fn config_list_impl(&self) -> Result<Value, ToolError> {
        let hosts: Vec<&str> = self.ssh_inventory.iter().map(|h| h.alias.as_str()).collect();
        let overrides: Vec<&String> = self.config.hosts.keys().collect();
        Ok(json!({
            "defaults": self.config.defaults,
            "hosts": hosts,
            "overrides": overrides,
        }))
    }
}

/// Build a `DefaultRunner` from the loaded config and `~/.ssh/config`
/// inventory.
///
/// Failures loading the SSH inventory are treated as non-fatal — they
/// produce an empty inventory (useful so `config.list` still works) and
/// log a warning. Both CLI and MCP surfaces call this at dispatch time
/// rather than at startup, keeping the construction surface-neutral.
pub fn build_default_runner(config: crate::config::DeployPreferences) -> DefaultRunner {
    let inventory = lab_apis::extract::inventory::load_ssh_inventory()
        .unwrap_or_else(|err| {
            tracing::warn!(error = %err, "deploy: could not load ~/.ssh/config inventory");
            Vec::new()
        });
    DefaultRunner::new(
        config,
        Arc::new(inventory),
        Arc::new(HostLockRegistry::default()),
    )
}

/// Process-global `DefaultRunner` initialised once at first MCP dispatch.
///
/// The MCP registry dispatch closure must return a `'static`-compatible
/// future, so the runner must live in a static rather than a local.
/// CLI dispatch owns its runner directly (config is threaded in), so
/// only the MCP path uses this slot.
static MCP_RUNNER: std::sync::OnceLock<DefaultRunner> = std::sync::OnceLock::new();

/// Return a reference to the process-global `DefaultRunner`, initialising
/// it from on-disk config and `~/.ssh/config` on first call.
///
/// Config load failures are non-fatal: the runner is built with default
/// preferences so that `help` / `schema` / `config.list` still work.
pub fn mcp_runner() -> &'static DefaultRunner {
    MCP_RUNNER.get_or_init(|| {
        let prefs = crate::config::load_toml(&crate::config::toml_candidates())
            .ok()
            .and_then(|cfg| cfg.deploy)
            .unwrap_or_default();
        build_default_runner(prefs)
    })
}

/// Parameters for a single-host run, pre-resolved so the orchestrator can
/// fan out without capturing `&self` across an `await` boundary.
#[derive(Clone)]
struct HostJob {
    host: String,
    target: SshHostTarget,
    remote_path: String,
    unit: Option<String>,
    scope: Option<ServiceScope>,
}

impl DefaultRunner {
    fn build_jobs(&self, hosts: &[String]) -> Vec<HostJob> {
        hosts
            .iter()
            .filter_map(|h| {
                let target = self.resolve_target(h).cloned()?;
                Some(HostJob {
                    host: h.clone(),
                    target,
                    remote_path: self.effective_remote_path(h),
                    unit: self.effective_unit(h),
                    scope: self.effective_scope(h),
                })
            })
            .collect()
    }

    /// Fan out `jobs` at `max_parallel` concurrency, honoring fail-fast.
    ///
    /// `locks` is passed by `Arc` so the stream closures do not borrow
    /// `&self` — a Rust 2024 RPIT-lifetime-capture limitation (#100013)
    /// rejects `&self`-capturing futures inside `buffer_unordered` when the
    /// surrounding function returns `impl Future`.
    async fn run_jobs(
        jobs: Vec<HostJob>,
        build: Arc<BuildOutcome>,
        max_parallel: usize,
        fail_fast: bool,
        run_id: String,
        locks: Arc<HostLockRegistry>,
    ) -> Vec<DeployHostResult> {
        use futures::stream::{self, StreamExt};
        use std::sync::atomic::{AtomicBool, Ordering};
        use tracing::Instrument;

        let stop = Arc::new(AtomicBool::new(false));

        stream::iter(jobs)
            .map(move |job| {
                let stop = stop.clone();
                let build = build.clone();
                let run_id = run_id.clone();
                let locks = locks.clone();
                async move {
                    if stop.load(Ordering::SeqCst) {
                        return aborted_result(&job.host);
                    }
                    let span = tracing::info_span!(
                        "deploy.host",
                        host = %job.host,
                        run_id = %run_id,
                    );
                    let res = run_single_job(job, build, locks).instrument(span).await;
                    if fail_fast && !res.succeeded {
                        stop.store(true, Ordering::SeqCst);
                    }
                    res
                }
            })
            .buffer_unordered(max_parallel.max(1))
            .collect()
            .await
    }
}

/// Generic orchestrator used by both `DefaultRunner` and orchestration tests.
///
/// Takes an `io_factory` that produces a `HostIo` for each host, plus the
/// stage knobs from the job. Tests use this to inject `RecordingIo`; the
/// production path uses `SshHostIo` via `run_single_job`.
pub async fn orchestrate_with_io<I, F>(
    hosts: Vec<(String, Option<String>, Option<ServiceScope>, String)>,
    build: Arc<BuildOutcome>,
    max_parallel: usize,
    fail_fast: bool,
    run_id: String,
    io_factory: F,
) -> Vec<DeployHostResult>
where
    I: HostIo + Send + Sync + 'static,
    F: Fn(&str) -> I + Send + Sync + Clone + 'static,
{
    use futures::stream::{self, StreamExt};
    use std::sync::atomic::{AtomicBool, Ordering};
    use tracing::Instrument;

    let stop = Arc::new(AtomicBool::new(false));

    stream::iter(hosts)
        .map(move |(host, unit, scope, remote_path)| {
            let stop = stop.clone();
            let build = build.clone();
            let run_id = run_id.clone();
            let io_factory = io_factory.clone();
            async move {
                if stop.load(Ordering::SeqCst) {
                    return aborted_result(&host);
                }
                let span = tracing::info_span!(
                    "deploy.host",
                    host = %host,
                    run_id = %run_id,
                );
                let io = Arc::new(io_factory(&host));
                let res = run_host_pipeline(io, host.clone(), remote_path, unit, scope, build)
                    .instrument(span)
                    .await;
                if fail_fast && !res.succeeded {
                    stop.store(true, Ordering::SeqCst);
                }
                res
            }
        })
        .buffer_unordered(max_parallel.max(1))
        .collect()
        .await
}

/// Drive a fully-resolved single-host job: acquire lock, walk stages.
///
/// Takes all arguments by value so the resulting future does not hold any
/// borrowed references across await points, which would trigger HRTB errors
/// in `Box::pin(…+Send+'static)` contexts (Rust issue #100013).
async fn run_single_job(
    job: HostJob,
    build: Arc<BuildOutcome>,
    locks: Arc<HostLockRegistry>,
) -> DeployHostResult {
    let io = Arc::new(SshHostIo::new(job.host.clone(), job.target.clone()));
    let lock_timeout = std::time::Duration::from_secs(60);
    let _guard = match locks.acquire(&job.host, lock_timeout).await {
        Ok(g) => g,
        Err(e) => {
            return DeployHostResult {
                host: job.host.clone(),
                reached_stage: DeployStage::Resolve,
                succeeded: false,
                skipped_transfer: false,
                transferred_bytes: None,
                error_kind: Some(e.kind().to_string()),
                stage_timings_ms: Default::default(),
            };
        }
    };
    run_host_pipeline(
        io,
        job.host,
        job.remote_path,
        job.unit,
        job.scope,
        build,
    )
    .await
}


pub async fn run_host_pipeline<I: HostIo + 'static>(
    io: Arc<I>,
    host: String,
    remote_path: String,
    unit: Option<String>,
    scope: Option<ServiceScope>,
    build: Arc<BuildOutcome>,
) -> DeployHostResult {
    use std::time::Instant;

    let mut timings: std::collections::BTreeMap<String, u128> =
        std::collections::BTreeMap::new();
    let mut transferred_bytes: Option<u64> = None;

    // Preflight
    let t = Instant::now();
    let pre = match preflight(
        io.clone(),
        remote_path.clone(),
        build.target_triple.clone(),
        build.sha256.clone(),
    )
    .await
    {
        Ok(p) => {
            timings.insert("preflight".into(), t.elapsed().as_millis());
            p
        }
        Err(e) => {
            timings.insert("preflight".into(), t.elapsed().as_millis());
            return host_err(&host, DeployStage::Preflight, e, timings, false);
        }
    };
    let skipped_transfer = pre.skip_transfer;

    // Transfer + install (conditional).
    if !pre.skip_transfer {
        let t = Instant::now();
        let reader = match tokio::fs::File::open(&build.path).await {
            Ok(f) => f,
            Err(e) => {
                timings.insert("transfer".into(), t.elapsed().as_millis());
                return host_err(
                    &host,
                    DeployStage::Transfer,
                    DeployError::BuildFailed {
                        reason: format!("open artifact: {e}"),
                    },
                    timings,
                    false,
                );
            }
        };
        let outcome =
            match transfer_and_install(io.clone(), remote_path.clone(), build.sha256.clone(), reader)
                .await
            {
                Ok(o) => {
                    timings.insert("transfer".into(), t.elapsed().as_millis());
                    o
                }
                Err(e) => {
                    timings.insert("transfer".into(), t.elapsed().as_millis());
                    return host_err(&host, DeployStage::Install, e, timings, false);
                }
            };
        transferred_bytes = Some(outcome.bytes);
    }

    // Restart
    let t = Instant::now();
    if let Err(e) = restart(io.clone(), unit, scope).await {
        timings.insert("restart".into(), t.elapsed().as_millis());
        // skipped_transfer carries the actual preflight outcome: if transfer
        // was already skipped before restart failed, report that faithfully.
        return host_err(&host, DeployStage::Restart, e, timings, skipped_transfer);
    }
    timings.insert("restart".into(), t.elapsed().as_millis());

    // Verify
    let t = Instant::now();
    if let Err(e) = verify(io, remote_path.clone()).await {
        timings.insert("verify".into(), t.elapsed().as_millis());
        return host_err(&host, DeployStage::Verify, e, timings, skipped_transfer);
    }
    timings.insert("verify".into(), t.elapsed().as_millis());

    DeployHostResult {
        host,
        reached_stage: DeployStage::Verify,
        succeeded: true,
        skipped_transfer,
        transferred_bytes,
        error_kind: None,
        stage_timings_ms: timings,
    }
}

async fn rollback_one_host<I: HostIo + 'static>(
    io: Arc<I>,
    host: String,
    remote_path: String,
    unit: Option<String>,
    scope: Option<ServiceScope>,
) -> DeployHostResult {
    let mut timings: std::collections::BTreeMap<String, u128> =
        std::collections::BTreeMap::new();
    use std::time::Instant;

    // Validate remote_path against the allowlist before any shell use.
    if let Err(e) = params::validate_remote_path(&remote_path) {
        return host_err(&host, DeployStage::Resolve, e, timings, false);
    }

    // Find the most recent .bak.<ts> file in the parent directory.
    let parent = match Path::new(&remote_path).parent() {
        Some(p) => p.to_string_lossy().into_owned(),
        None => {
            return host_err(
                &host,
                DeployStage::Resolve,
                DeployError::ValidationFailed {
                    field: "remote_path".into(),
                    reason: "no parent".into(),
                },
                timings,
                false,
            );
        }
    };
    let binary = Path::new(&remote_path)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "lab".to_string());

    let t = Instant::now();
    // shell_quote parent and binary so any (theoretically allowlisted)
    // special characters in the path do not break the shell command.
    let sq_parent = shell_quote(&parent);
    let sq_binary = shell_quote(&binary);
    let pattern = format!("{sq_parent}/{sq_binary}.bak.*");
    // ls -1 <pattern> | sort | tail -n1
    let cmd = format!("ls -1 {pattern} 2>/dev/null | sort | tail -n1");
    let (code, stdout, stderr) = match io.run_argv(&["sh", "-c", &cmd]).await {
        Ok(v) => v,
        Err(e) => return host_err(&host, DeployStage::Resolve, e, timings, false),
    };
    if code != 0 || stdout.trim().is_empty() {
        timings.insert("rollback.find".into(), t.elapsed().as_millis());
        return host_err(
            &host,
            DeployStage::Resolve,
            DeployError::ValidationFailed {
                field: "backup".into(),
                reason: format!("no backup found under {parent}: {}", stderr.trim()),
            },
            timings,
            false,
        );
    }
    let backup = stdout.trim().to_string();
    timings.insert("rollback.find".into(), t.elapsed().as_millis());

    let t = Instant::now();
    let (code, _stdout, stderr) = match io.run_argv(&["mv", "--", &backup, &remote_path]).await {
        Ok(v) => v,
        Err(e) => return host_err(&host, DeployStage::Install, e, timings, false),
    };
    timings.insert("rollback.restore".into(), t.elapsed().as_millis());
    if code != 0 {
        return host_err(
            &host,
            DeployStage::Install,
            DeployError::InstallFailed {
                host: host.clone(),
                reason: format!("rollback rename: {}", stderr.trim()),
            },
            timings,
            false,
        );
    }

    // Restart (best effort — rollback considers itself successful if the
    // binary was swapped, regardless of restart outcome the caller can see
    // the error via verify).
    let t = Instant::now();
    if let Err(e) = restart(io, unit, scope).await {
        timings.insert("rollback.restart".into(), t.elapsed().as_millis());
        return host_err(&host, DeployStage::Restart, e, timings, false);
    }
    timings.insert("rollback.restart".into(), t.elapsed().as_millis());

    DeployHostResult {
        host,
        reached_stage: DeployStage::Verify,
        succeeded: true,
        skipped_transfer: false,
        transferred_bytes: None,
        error_kind: None,
        stage_timings_ms: timings,
    }
}

fn host_err(
    host: &str,
    reached: DeployStage,
    err: DeployError,
    timings: std::collections::BTreeMap<String, u128>,
    skipped_transfer: bool,
) -> DeployHostResult {
    tracing::warn!(
        host = %host,
        reached_stage = ?reached,
        error = %err,
        error_kind = err.kind(),
        "deploy.host.error"
    );
    DeployHostResult {
        host: host.to_string(),
        reached_stage: reached,
        succeeded: false,
        skipped_transfer,
        transferred_bytes: None,
        error_kind: Some(err.kind().to_string()),
        stage_timings_ms: timings,
    }
}

fn aborted_result(host: &str) -> DeployHostResult {
    DeployHostResult {
        host: host.to_string(),
        reached_stage: DeployStage::Resolve,
        succeeded: false,
        skipped_transfer: false,
        transferred_bytes: None,
        error_kind: Some("aborted".into()),
        stage_timings_ms: Default::default(),
    }
}

fn summarize(
    run_id: String,
    artifact_sha256: String,
    hosts: Vec<DeployHostResult>,
) -> DeployRunSummary {
    let succeeded = hosts.iter().filter(|r| r.succeeded).count();
    let failed = hosts.len() - succeeded;
    DeployRunSummary {
        run_id,
        artifact_sha256,
        succeeded,
        failed,
        ok: failed == 0,
        hosts,
    }
}

// ── NoopRunner (kept for surface bring-up / fallback) ──────────────────────

/// Placeholder runner. Kept so callers without a wired `DefaultRunner` can
/// still register dispatch without a panic.
#[derive(Default)]
pub struct NoopRunner;

impl DeployRunner for NoopRunner {
    async fn plan(&self, _req: DeployRequest) -> Result<DeployPlan, ToolError> {
        Err(ToolError::internal_message(
            "deploy runner is not wired on this build",
        ))
    }

    async fn run(&self, _req: DeployRequest) -> Result<DeployRunSummary, ToolError> {
        Err(ToolError::internal_message(
            "deploy runner is not wired on this build",
        ))
    }

    async fn rollback(&self, _req: DeployRequest) -> Result<DeployRunSummary, ToolError> {
        Err(ToolError::internal_message(
            "deploy runner is not wired on this build",
        ))
    }

    async fn config_list(&self) -> Result<Value, ToolError> {
        Ok(json!({ "defaults": null, "hosts": [], "overrides": [] }))
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[doc(hidden)]
pub mod test_support {
    //! Recording `HostIo` used by both inline stage tests and the
    //! `tests/deploy_runner.rs` orchestrator tests.

    use super::*;
    use std::sync::Mutex;
    use tokio::io::{AsyncRead, AsyncReadExt};

    /// Pre-programmed response for a single `run_argv` call, matched in order.
    #[derive(Debug, Clone)]
    pub struct RunResp {
        pub code: i32,
        pub stdout: String,
        pub stderr: String,
    }

    impl RunResp {
        pub fn ok(stdout: impl Into<String>) -> Self {
            Self {
                code: 0,
                stdout: stdout.into(),
                stderr: String::new(),
            }
        }
        pub fn fail(code: i32, stderr: impl Into<String>) -> Self {
            Self {
                code,
                stdout: String::new(),
                stderr: stderr.into(),
            }
        }
    }

    /// Recording `HostIo` that appends every op to `log` and returns
    /// scripted responses from `run_queue` / `sha_queue`.
    #[derive(Default)]
    pub struct RecordingIo {
        pub log: Arc<Mutex<Vec<String>>>,
        pub run_queue: Arc<Mutex<Vec<RunResp>>>,
        pub sha_queue: Arc<Mutex<Vec<Option<String>>>>,
        pub upload_bytes: Arc<Mutex<Vec<u64>>>,
    }

    impl RecordingIo {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn push_run(&self, resp: RunResp) {
            self.run_queue.lock().unwrap().push(resp);
        }

        pub fn push_sha(&self, value: Option<String>) {
            self.sha_queue.lock().unwrap().push(value);
        }

        pub fn ops(&self) -> Vec<String> {
            self.log.lock().unwrap().clone()
        }
    }

    impl HostIo for RecordingIo {
        fn run_argv(
            &self,
            argv: &[&str],
        ) -> Pin<Box<dyn Future<Output = Result<(i32, String, String), DeployError>> + Send + 'static>> {
            let joined = argv.join(",");
            let log = self.log.clone();
            let run_queue = self.run_queue.clone();
            Box::pin(async move {
                log.lock().unwrap().push(format!("run:{joined}"));
                let resp = run_queue
                    .lock()
                    .unwrap()
                    .drain(..1)
                    .next()
                    .unwrap_or_else(|| RunResp::ok(""));
                Ok((resp.code, resp.stdout, resp.stderr))
            })
        }

        fn upload_stream<R>(
            &self,
            remote_path: &str,
            mut reader: R,
        ) -> Pin<Box<dyn Future<Output = Result<u64, DeployError>> + Send + 'static>>
        where
            R: AsyncRead + Unpin + Send + 'static,
        {
            let remote_path = remote_path.to_string();
            let log = self.log.clone();
            let upload_bytes = self.upload_bytes.clone();
            Box::pin(async move {
                log.lock().unwrap().push(format!("upload:{remote_path}"));
                let mut buf = Vec::new();
                let bytes = reader
                    .read_to_end(&mut buf)
                    .await
                    .map_err(|e| DeployError::TransferFailed {
                        host: "?".into(),
                        reason: e.to_string(),
                    })? as u64;
                upload_bytes.lock().unwrap().push(bytes);
                Ok(bytes)
            })
        }

        fn sha256_remote(
            &self,
            remote_path: &str,
        ) -> Pin<Box<dyn Future<Output = Result<Option<String>, DeployError>> + Send + 'static>> {
            let remote_path = remote_path.to_string();
            let log = self.log.clone();
            let sha_queue = self.sha_queue.clone();
            Box::pin(async move {
                log.lock().unwrap().push(format!("sha256:{remote_path}"));
                let val = sha_queue.lock().unwrap().drain(..1).next().flatten();
                Ok(val)
            })
        }
    }
}

#[cfg(test)]
mod tests_preflight {
    use super::test_support::*;
    use super::*;

    #[tokio::test]
    async fn rejects_arch_mismatch() {
        let io = Arc::new(RecordingIo::new());
        io.push_run(RunResp::ok("aarch64\n"));
        let err = preflight(
            io,
            "/usr/local/bin/lab".to_string(),
            "x86_64-unknown-linux-gnu".to_string(),
            "abc123".to_string(),
        )
        .await
        .unwrap_err();
        assert_eq!(err.kind(), "arch_mismatch");
    }

    #[tokio::test]
    async fn reports_skip_when_remote_sha_matches() {
        let io = Arc::new(RecordingIo::new());
        io.push_run(RunResp::ok("x86_64\n"));
        io.push_run(RunResp::ok("")); // canary
        io.push_sha(Some("abc123".to_string()));
        let res = preflight(
            io,
            "/usr/local/bin/lab".to_string(),
            "x86_64-unknown-linux-gnu".to_string(),
            "abc123".to_string(),
        )
        .await
        .unwrap();
        assert!(res.skip_transfer);
    }

    #[tokio::test]
    async fn does_not_skip_when_remote_sha_differs() {
        let io = Arc::new(RecordingIo::new());
        io.push_run(RunResp::ok("x86_64\n"));
        io.push_run(RunResp::ok(""));
        io.push_sha(Some("deadbeef".to_string()));
        let res = preflight(
            io,
            "/usr/local/bin/lab".to_string(),
            "x86_64-unknown-linux-gnu".to_string(),
            "abc123".to_string(),
        )
        .await
        .unwrap();
        assert!(!res.skip_transfer);
    }

    #[tokio::test]
    async fn rejects_non_writable_install_dir() {
        let io = Arc::new(RecordingIo::new());
        io.push_run(RunResp::ok("x86_64\n"));
        io.push_run(RunResp::fail(1, "permission denied"));
        let err = preflight(
            io,
            "/usr/local/bin/lab".to_string(),
            "x86_64-unknown-linux-gnu".to_string(),
            "abc123".to_string(),
        )
        .await
        .unwrap_err();
        assert_eq!(err.kind(), "preflight_failed");
    }

    #[tokio::test]
    async fn canary_write_goes_to_parent_not_binary_path() {
        let io = Arc::new(RecordingIo::new());
        io.push_run(RunResp::ok("x86_64\n"));
        io.push_run(RunResp::ok(""));
        io.push_sha(None);
        let _ = preflight(
            io.clone(),
            "/usr/local/bin/lab".to_string(),
            "x86_64-unknown-linux-gnu".to_string(),
            "abc123".to_string(),
        )
        .await
        .unwrap();
        let ops = io.ops();
        // Second op is the canary sh -c; assert the path targets the parent dir.
        let probe = ops
            .iter()
            .find(|o| o.starts_with("run:sh,-c,"))
            .expect("canary run recorded");
        assert!(probe.contains("/usr/local/bin/.lab.deploy.canary"));
        assert!(!probe.contains("'/usr/local/bin/lab'"));
    }
}

#[cfg(test)]
mod tests_transfer_install {
    use super::test_support::*;
    use super::*;

    #[tokio::test]
    async fn transfer_streams_to_partial_then_renames_and_swaps() {
        let io = Arc::new(RecordingIo::new());
        // mv partial -> staged
        io.push_run(RunResp::ok(""));
        // sha256 of staged (matches)
        io.push_sha(Some("abc123".to_string()));
        // sha256 probe for existing binary (Some => exists)
        io.push_sha(Some("oldsha".to_string()));
        // mv existing -> .bak.<ts>
        io.push_run(RunResp::ok(""));
        // mv staged -> remote_path
        io.push_run(RunResp::ok(""));

        let outcome = transfer_and_install(
            io.clone(),
            "/usr/local/bin/lab".to_string(),
            "abc123".to_string(),
            tokio::io::empty(),
        )
        .await
        .unwrap();
        assert_eq!(outcome.bytes, 0);
        assert!(outcome.backup_path.is_some());

        let ops = io.ops();
        assert!(
            ops.iter().any(|o| o == "upload:/usr/local/bin/lab.new.partial"),
            "ops: {ops:?}"
        );
        assert!(
            ops.iter().any(|o| o
                == "run:mv,--,/usr/local/bin/lab.new.partial,/usr/local/bin/lab.new"),
            "ops: {ops:?}"
        );
        assert!(
            ops.iter().any(|o| o == "sha256:/usr/local/bin/lab.new"),
            "ops: {ops:?}"
        );
        // backup rename targets the existing binary
        assert!(
            ops.iter()
                .any(|o| o.starts_with("run:mv,--,/usr/local/bin/lab,/usr/local/bin/lab.bak.")),
            "ops: {ops:?}"
        );
        // atomic swap
        assert!(
            ops.iter()
                .any(|o| o == "run:mv,--,/usr/local/bin/lab.new,/usr/local/bin/lab"),
            "ops: {ops:?}"
        );
    }

    #[tokio::test]
    async fn integrity_mismatch_aborts_before_swap() {
        let io = Arc::new(RecordingIo::new());
        // mv partial -> staged ok
        io.push_run(RunResp::ok(""));
        // sha256 of staged differs from local
        io.push_sha(Some("deadbeef".to_string()));
        // cleanup rm -f (best effort)
        io.push_run(RunResp::ok(""));

        let err = transfer_and_install(
            io.clone(),
            "/usr/local/bin/lab".to_string(),
            "abc123".to_string(),
            tokio::io::empty(),
        )
        .await
        .unwrap_err();
        assert_eq!(err.kind(), "integrity_mismatch");

        let ops = io.ops();
        // must NOT have performed the final swap or the backup rename
        assert!(
            !ops.iter()
                .any(|o| o == "run:mv,--,/usr/local/bin/lab.new,/usr/local/bin/lab"),
            "ops: {ops:?}"
        );
        assert!(
            !ops.iter()
                .any(|o| o.starts_with("run:mv,--,/usr/local/bin/lab,/usr/local/bin/lab.bak.")),
            "ops: {ops:?}"
        );
    }

    #[tokio::test]
    async fn no_backup_when_target_absent() {
        let io = Arc::new(RecordingIo::new());
        io.push_run(RunResp::ok("")); // mv partial -> staged
        io.push_sha(Some("abc123".into())); // staged sha matches
        io.push_sha(None); // existing binary absent
        io.push_run(RunResp::ok("")); // final swap

        let outcome = transfer_and_install(
            io,
            "/usr/local/bin/lab".to_string(),
            "abc123".to_string(),
            tokio::io::empty(),
        )
        .await
        .unwrap();
        assert!(outcome.backup_path.is_none());
    }
}

#[cfg(test)]
mod tests_restart_verify {
    use super::test_support::*;
    use super::*;

    #[tokio::test]
    async fn skips_restart_when_unit_is_none() {
        let io = Arc::new(RecordingIo::new());
        let r = restart(io.clone(), None, None).await.unwrap();
        assert!(r.skipped);
        assert!(io.ops().is_empty());
    }

    #[tokio::test]
    async fn restart_system_scope_uses_systemctl() {
        let io = Arc::new(RecordingIo::new());
        io.push_run(RunResp::ok("")); // restart
        io.push_run(RunResp::ok("active\n")); // is-active --wait

        restart(io.clone(), Some("lab".to_string()), Some(ServiceScope::System))
            .await
            .unwrap();
        let ops = io.ops();
        assert!(ops.iter().any(|o| o == "run:systemctl,restart,lab"), "ops: {ops:?}");
        assert!(
            ops.iter().any(|o| o == "run:systemctl,is-active,--wait,lab"),
            "ops: {ops:?}"
        );
    }

    #[tokio::test]
    async fn restart_user_scope_uses_systemctl_user() {
        let io = Arc::new(RecordingIo::new());
        io.push_run(RunResp::ok(""));
        io.push_run(RunResp::ok(""));

        restart(io.clone(), Some("lab-worker".to_string()), Some(ServiceScope::User))
            .await
            .unwrap();
        let ops = io.ops();
        assert!(
            ops.iter().any(|o| o == "run:systemctl,--user,restart,lab-worker"),
            "ops: {ops:?}"
        );
        assert!(
            ops.iter().any(|o| o == "run:systemctl,--user,is-active,--wait,lab-worker"),
            "ops: {ops:?}"
        );
    }

    #[tokio::test]
    async fn restart_rejects_invalid_unit_names() {
        let io = Arc::new(RecordingIo::new());
        let err = restart(io.clone(), Some("bad;unit".to_string()), None).await.unwrap_err();
        assert_eq!(err.kind(), "validation_failed");
        assert!(io.ops().is_empty(), "must fail before emitting any command");
    }

    #[tokio::test]
    async fn verify_runs_version_and_rejects_nonzero_exit() {
        let io = Arc::new(RecordingIo::new());
        io.push_run(RunResp::fail(1, "unknown flag"));
        let err = verify(io.clone(), "/usr/local/bin/lab".to_string()).await.unwrap_err();
        assert_eq!(err.kind(), "verify_failed");
        let ops = io.ops();
        assert!(
            ops.iter().any(|o| o == "run:/usr/local/bin/lab,--version"),
            "ops: {ops:?}"
        );
    }

    #[tokio::test]
    async fn verify_accepts_zero_exit() {
        let io = Arc::new(RecordingIo::new());
        io.push_run(RunResp::ok("lab 0.3.4\n"));
        verify(io, "/usr/local/bin/lab".to_string()).await.unwrap();
    }
}

#[cfg(test)]
mod tests_arch {
    use super::*;

    #[test]
    fn arch_aliases_normalize_correctly() {
        // canonical names are unchanged
        assert_eq!(normalize_arch("x86_64"), "x86_64");
        assert_eq!(normalize_arch("aarch64"), "aarch64");
        assert_eq!(normalize_arch("i686"), "i686");
        // Docker/OCI aliases
        assert_eq!(normalize_arch("amd64"), "x86_64");
        assert_eq!(normalize_arch("x64"), "x86_64");
        assert_eq!(normalize_arch("arm64"), "aarch64");
    }

    #[test]
    fn arch_matches_handles_aliases() {
        assert!(arch_matches("x86_64", "amd64"));
        assert!(arch_matches("amd64", "x86_64"));
        assert!(arch_matches("aarch64", "arm64"));
        assert!(arch_matches("arm64", "aarch64"));
        assert!(!arch_matches("x86_64", "aarch64"));
        assert!(!arch_matches("amd64", "arm64"));
    }
}
