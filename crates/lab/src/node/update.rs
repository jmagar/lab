use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result, anyhow, bail};
use serde::Serialize;
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::config::{DeployPreferences, LabConfig, RestartModel, ServiceScope};
use crate::dispatch::deploy::build::{BuildOutcome, build_release};
use crate::dispatch::deploy::host_io::{HostIo, SshHostIo};
use crate::dispatch::deploy::params::validate_remote_path;
use crate::dispatch::deploy::runner::build_default_runner;
use crate::dispatch::deploy::ssh_session::{SshHostTarget, shell_quote};
use crate::dispatch::deploy::stages::{preflight, restart, transfer_and_install, verify};
use crate::node::identity::resolve_local_hostname;
use crate::node::master_client::MasterClient;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
enum UpdateTargetKind {
    Remote,
    LocalController,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
enum RestartSelection {
    SystemService,
    UserService,
    WrapperCommand,
}

#[derive(Debug, Clone, Serialize)]
struct UpdateTargetResult {
    target: String,
    kind: UpdateTargetKind,
    node_id: Option<String>,
    connected: Option<bool>,
    controller_health_ok: Option<bool>,
    skipped_transfer: bool,
    ok: bool,
    failed_stage: Option<String>,
    stages_ms: BTreeMap<String, u128>,
    error: Option<String>,
}

#[derive(Debug, Clone)]
struct RemoteTarget {
    alias: String,
    ssh: SshHostTarget,
}

#[derive(Debug, Clone)]
struct LocalTarget {
    identity: String,
}

#[derive(Debug, Clone)]
struct ResolvedTargets {
    remote: Vec<RemoteTarget>,
    local_controller: Option<LocalTarget>,
}

#[derive(Debug, Clone)]
struct EffectiveTargetConfig {
    install_path: String,
    restart: Option<RestartModel>,
}

pub async fn run_update(
    config: &LabConfig,
    explicit_targets: Vec<String>,
    all: bool,
) -> Result<serde_json::Value> {
    let local_host = resolve_local_hostname().context("resolve local hostname for nodes update")?;
    let controller_host = controller_host(config, &local_host);
    let resolved = resolve_targets(config, &local_host, &controller_host, explicit_targets, all)?;
    let artifact = Arc::new(build_release().await?);

    let controller_client =
        MasterClient::from_config(config, None).context("build controller verification client")?;

    let mut results = Vec::new();
    for target in resolved.remote {
        let target_config = effective_target_config(config, &target.alias);
        let io = Arc::new(SshHostIo::new(target.alias.clone(), target.ssh.clone()));
        results.push(
            run_remote_target(
                io,
                target.alias,
                controller_host.clone(),
                target_config,
                artifact.clone(),
                &controller_client,
            )
            .await,
        );
    }

    if let Some(local_target) = resolved.local_controller {
        let target_config = effective_target_config(config, &local_target.identity);
        results.push(
            run_local_controller(
                local_target.identity,
                controller_host,
                target_config,
                artifact.clone(),
            )
            .await,
        );
    }

    let ok = results.iter().all(|result| result.ok);
    Ok(json!({
        "ok": ok,
        "artifact": {
            "path": artifact.path.clone(),
            "sha256": artifact.sha256.clone(),
            "size_bytes": artifact.size_bytes,
            "target_triple": artifact.target_triple.clone(),
        },
        "results": results,
    }))
}

fn controller_host(config: &LabConfig, local_host: &str) -> String {
    config
        .controller_host()
        .and_then(normalize_host_identifier)
        .unwrap_or_else(|| normalize_host_identifier(local_host).unwrap_or_else(|| "localhost".into()))
}

fn resolve_targets(
    config: &LabConfig,
    local_host: &str,
    controller_host: &str,
    explicit_targets: Vec<String>,
    all: bool,
) -> Result<ResolvedTargets> {
    let runner = build_default_runner(config.deploy.clone().unwrap_or_default());
    let inventory = runner.ssh_inventory.clone();
    let is_local_controller = hosts_match(local_host, controller_host);

    let mut remote = Vec::new();
    let mut local_controller = None;

    if all {
        for target in inventory.iter() {
            if is_local_controller && ssh_target_matches_local(target, local_host, controller_host) {
                local_controller = Some(LocalTarget {
                    identity: controller_host.to_string(),
                });
                continue;
            }
            remote.push(RemoteTarget {
                alias: target.alias.clone(),
                ssh: target.clone(),
            });
        }
    } else {
        for requested in explicit_targets {
            if is_local_controller && hosts_match(&requested, controller_host) {
                local_controller = Some(LocalTarget {
                    identity: controller_host.to_string(),
                });
                continue;
            }

            let target = inventory
                .iter()
                .find(|target| target.alias == requested)
                .cloned()
                .ok_or_else(|| anyhow!("unknown node target `{requested}`"))?;

            if is_local_controller && ssh_target_matches_local(&target, local_host, controller_host) {
                local_controller = Some(LocalTarget {
                    identity: controller_host.to_string(),
                });
                continue;
            }

            remote.push(RemoteTarget {
                alias: target.alias.clone(),
                ssh: target,
            });
        }
    }

    if is_local_controller && local_controller.is_none() {
        local_controller = Some(LocalTarget {
            identity: controller_host.to_string(),
        });
    }

    if remote.is_empty() && local_controller.is_none() {
        bail!("no node update targets resolved");
    }

    Ok(ResolvedTargets {
        remote,
        local_controller,
    })
}

fn effective_target_config(config: &LabConfig, target: &str) -> EffectiveTargetConfig {
    let deploy = config.deploy.clone().unwrap_or_default();
    let host = deploy.hosts.get(target);
    let defaults = deploy.defaults.as_ref();

    let install_path = host
        .and_then(|entry| entry.remote_path.clone())
        .or_else(|| defaults.and_then(|entry| entry.remote_path.clone()))
        .unwrap_or_else(|| "/usr/local/bin/lab".to_string());

    let restart = host
        .and_then(|entry| entry.restart.clone())
        .or_else(|| defaults.and_then(|entry| entry.restart.clone()))
        .or_else(|| legacy_restart_model(&deploy, target));

    EffectiveTargetConfig {
        install_path,
        restart,
    }
}

fn legacy_restart_model(deploy: &DeployPreferences, target: &str) -> Option<RestartModel> {
    let host = deploy.hosts.get(target);
    let service = match host.and_then(|entry| entry.service.as_deref()) {
        Some("") => return None,
        Some(service) => Some(service.to_string()),
        None => deploy
            .defaults
            .as_ref()
            .and_then(|entry| entry.service.clone())
            .filter(|service| !service.is_empty()),
    }?;

    let scope = host
        .and_then(|entry| entry.service_scope)
        .or_else(|| deploy.defaults.as_ref().and_then(|entry| entry.service_scope))
        .unwrap_or(ServiceScope::System);

    Some(match scope {
        ServiceScope::System => RestartModel::SystemService { service },
        ServiceScope::User => RestartModel::UserService { service },
    })
}

async fn run_remote_target<I: HostIo + 'static>(
    io: Arc<I>,
    alias: String,
    controller_host: String,
    target_config: EffectiveTargetConfig,
    artifact: Arc<BuildOutcome>,
    controller_client: &MasterClient,
) -> UpdateTargetResult {
    let mut stages_ms = BTreeMap::new();
    let mut skipped_transfer = false;

    let resolved_node_id = match resolve_remote_node_id(io.clone()).await {
        Ok(node_id) => node_id,
        Err(error) => {
            return failed_result(
                alias,
                UpdateTargetKind::Remote,
                None,
                false,
                "resolve".into(),
                stages_ms,
                error.to_string(),
            );
        }
    };

    let preflight_started = Instant::now();
    let preflight_result = preflight(
        io.clone(),
        target_config.install_path.clone(),
        artifact.target_triple.clone(),
        artifact.sha256.clone(),
    )
    .await;
    stages_ms.insert("preflight".into(), preflight_started.elapsed().as_millis());
    let preflight_result = match preflight_result {
        Ok(result) => result,
        Err(error) => {
            return failed_result(
                alias,
                UpdateTargetKind::Remote,
                Some(resolved_node_id),
                false,
                "preflight".into(),
                stages_ms,
                error.to_string(),
            );
        }
    };
    skipped_transfer = preflight_result.skip_transfer;

    if !preflight_result.skip_transfer {
        let transfer_started = Instant::now();
        let file = match tokio::fs::File::open(&artifact.path).await {
            Ok(file) => file,
            Err(error) => {
                return failed_result(
                    alias,
                    UpdateTargetKind::Remote,
                    Some(resolved_node_id),
                    false,
                    "transfer".into(),
                    stages_ms,
                    format!("open artifact: {error}"),
                );
            }
        };
        let transfer_result = transfer_and_install(
            io.clone(),
                target_config.install_path.clone(),
                artifact.sha256.clone(),
                file,
            )
        .await;
        stages_ms.insert("transfer".into(), transfer_started.elapsed().as_millis());
        if let Err(error) = transfer_result {
            return failed_result(
                alias.clone(),
                UpdateTargetKind::Remote,
                Some(resolved_node_id.clone()),
                false,
                "transfer".into(),
                stages_ms,
                error.to_string(),
            );
        }
    }

    let normalize_started = Instant::now();
    if let Err(error) =
        normalize_remote_runtime(io.clone(), &resolved_node_id, &controller_host).await
    {
        stages_ms.insert("normalize".into(), normalize_started.elapsed().as_millis());
        return failed_result(
            alias.clone(),
            UpdateTargetKind::Remote,
            Some(resolved_node_id.clone()),
            skipped_transfer,
            "normalize".into(),
            stages_ms,
            error.to_string(),
        );
    }
    stages_ms.insert("normalize".into(), normalize_started.elapsed().as_millis());

    let restart_started = Instant::now();
    if let Err(error) = restart_target(io.clone(), target_config.restart.as_ref()).await {
        stages_ms.insert("restart".into(), restart_started.elapsed().as_millis());
        return failed_result(
            alias.clone(),
            UpdateTargetKind::Remote,
            Some(resolved_node_id.clone()),
            skipped_transfer,
            "restart".into(),
            stages_ms,
            error.to_string(),
        );
    }
    stages_ms.insert("restart".into(), restart_started.elapsed().as_millis());

    let verify_started = Instant::now();
    if let Err(error) = verify(io.clone(), target_config.install_path.clone()).await {
        stages_ms.insert("verify".into(), verify_started.elapsed().as_millis());
        return failed_result(
            alias.clone(),
            UpdateTargetKind::Remote,
            Some(resolved_node_id.clone()),
            skipped_transfer,
            "verify".into(),
            stages_ms,
            error.to_string(),
        );
    }
    stages_ms.insert("verify".into(), verify_started.elapsed().as_millis());

    let controller_started = Instant::now();
    let connected = controller_client
        .node_connected(&resolved_node_id)
        .await
        .unwrap_or(false);
    stages_ms.insert("controller_verify".into(), controller_started.elapsed().as_millis());
    if !connected {
        return failed_result(
            alias.clone(),
            UpdateTargetKind::Remote,
            Some(resolved_node_id.clone()),
            skipped_transfer,
            "controller_verify".into(),
            stages_ms,
            format!("controller did not report node `{}` as connected", resolved_node_id),
        );
    }

    UpdateTargetResult {
        target: alias,
        kind: UpdateTargetKind::Remote,
        node_id: Some(resolved_node_id),
        connected: Some(true),
        controller_health_ok: None,
        skipped_transfer,
        ok: true,
        failed_stage: None,
        stages_ms,
        error: None,
    }
}

async fn run_local_controller(
    identity: String,
    _controller_host: String,
    target_config: EffectiveTargetConfig,
    artifact: Arc<BuildOutcome>,
) -> UpdateTargetResult {
    let mut stages_ms = BTreeMap::new();
    let install_path = PathBuf::from(&target_config.install_path);

    let install_started = Instant::now();
    if let Err(error) = install_local_artifact(&artifact.path, &install_path).await {
        stages_ms.insert("install".into(), install_started.elapsed().as_millis());
        return failed_result(
            identity.clone(),
            UpdateTargetKind::LocalController,
            None,
            false,
            "install".into(),
            stages_ms,
            error.to_string(),
        );
    }
    stages_ms.insert("install".into(), install_started.elapsed().as_millis());

    let normalize_started = Instant::now();
    if let Err(error) = normalize_local_runtime(&identity).await {
        stages_ms.insert("normalize".into(), normalize_started.elapsed().as_millis());
        return failed_result(
            identity.clone(),
            UpdateTargetKind::LocalController,
            None,
            false,
            "normalize".into(),
            stages_ms,
            error.to_string(),
        );
    }
    stages_ms.insert("normalize".into(), normalize_started.elapsed().as_millis());

    let restart_started = Instant::now();
    if let Err(error) = restart_local_target(target_config.restart.as_ref()).await {
        stages_ms.insert("restart".into(), restart_started.elapsed().as_millis());
        return failed_result(
            identity.clone(),
            UpdateTargetKind::LocalController,
            None,
            false,
            "restart".into(),
            stages_ms,
            error.to_string(),
        );
    }
    stages_ms.insert("restart".into(), restart_started.elapsed().as_millis());

    let health_started = Instant::now();
    if let Err(error) = verify_local_health().await {
        stages_ms.insert("health".into(), health_started.elapsed().as_millis());
        return failed_result(
            identity.clone(),
            UpdateTargetKind::LocalController,
            None,
            false,
            "health".into(),
            stages_ms,
            error.to_string(),
        );
    }
    stages_ms.insert("health".into(), health_started.elapsed().as_millis());

    UpdateTargetResult {
        target: identity.clone(),
        kind: UpdateTargetKind::LocalController,
        node_id: Some(identity),
        connected: None,
        controller_health_ok: Some(true),
        skipped_transfer: false,
        ok: true,
        failed_stage: None,
        stages_ms,
        error: None,
    }
}

async fn resolve_remote_node_id<I: HostIo + 'static>(io: Arc<I>) -> Result<String> {
    let (code, stdout, stderr) = io.run_argv(&["hostname"]).await?;
    if code != 0 {
        bail!("resolve remote hostname failed: {}", stderr.trim());
    }
    normalize_host_identifier(stdout.trim())
        .ok_or_else(|| anyhow!("remote hostname command returned an empty identifier"))
}

async fn normalize_remote_runtime<I: HostIo + 'static>(
    io: Arc<I>,
    _node_id: &str,
    controller_host: &str,
) -> Result<()> {
    let home_dir = remote_home_dir(io.clone()).await?;
    let lab_dir = format!("{home_dir}/.lab");
    let config_path = format!("{lab_dir}/config.toml");
    let current = read_remote_file(io.clone(), &config_path).await.unwrap_or_default();

    let mut config = if current.trim().is_empty() {
        LabConfig::default()
    } else {
        toml::from_str::<LabConfig>(&current).unwrap_or_default()
    };
    config.device = None;
    config.node = Some(crate::config::NodePreferences {
        controller: Some(controller_host.to_string()),
        ..Default::default()
    });

    let rendered = toml::to_string_pretty(&config).context("serialize normalized node config")?;
    write_remote_file(io.clone(), &lab_dir, &config_path, &rendered).await?;

    let device_token = format!("{lab_dir}/device-token");
    let device_enrollments = format!("{lab_dir}/device-enrollments.json");
    let (code, _stdout, stderr) = io
        .run_argv(&["rm", "-f", "--", &device_token, &device_enrollments])
        .await?;
    if code != 0 {
        bail!("remove legacy runtime files failed: {}", stderr.trim());
    }
    Ok(())
}

async fn normalize_local_runtime(controller_host: &str) -> Result<()> {
    let home_dir = current_home_dir();
    let lab_dir = home_dir.join(".lab");
    tokio::fs::create_dir_all(&lab_dir)
        .await
        .with_context(|| format!("create {}", lab_dir.display()))?;
    let config_path = lab_dir.join("config.toml");
    let current = tokio::fs::read_to_string(&config_path).await.unwrap_or_default();
    let mut config = if current.trim().is_empty() {
        LabConfig::default()
    } else {
        toml::from_str::<LabConfig>(&current).unwrap_or_default()
    };
    config.device = None;
    config.node = Some(crate::config::NodePreferences {
        controller: Some(controller_host.to_string()),
        ..Default::default()
    });
    let rendered = toml::to_string_pretty(&config).context("serialize local normalized config")?;
    tokio::fs::write(&config_path, rendered)
        .await
        .with_context(|| format!("write {}", config_path.display()))?;

    let _ = tokio::fs::remove_file(lab_dir.join("device-token")).await;
    let _ = tokio::fs::remove_file(lab_dir.join("device-enrollments.json")).await;
    Ok(())
}

async fn remote_home_dir<I: HostIo + 'static>(io: Arc<I>) -> Result<String> {
    let (code, stdout, stderr) = io.run_argv(&["sh", "-c", "printf %s \"$HOME\""]).await?;
    if code != 0 {
        bail!("resolve remote home failed: {}", stderr.trim());
    }
    let home = stdout.trim();
    if home.is_empty() {
        bail!("remote home directory is empty");
    }
    Ok(home.to_string())
}

async fn read_remote_file<I: HostIo + 'static>(io: Arc<I>, path: &str) -> Result<String> {
    let quoted = shell_quote(path);
    let command = format!("if [ -f {quoted} ]; then cat {quoted}; fi");
    let (code, stdout, stderr) = io.run_argv(&["sh", "-c", &command]).await?;
    if code != 0 {
        bail!("read remote file `{path}` failed: {}", stderr.trim());
    }
    Ok(stdout)
}

async fn write_remote_file<I: HostIo + 'static>(
    io: Arc<I>,
    lab_dir: &str,
    path: &str,
    contents: &str,
) -> Result<()> {
    let marker = "__LAB_NODE_CONFIG__";
    let quoted_dir = shell_quote(lab_dir);
    let quoted_tmp = shell_quote(&format!("{path}.tmp"));
    let quoted_path = shell_quote(path);
    let command = format!(
        "mkdir -p {quoted_dir}\ncat > {quoted_tmp} <<'{marker}'\n{contents}\n{marker}\nmv -- {quoted_tmp} {quoted_path}"
    );
    let (code, _stdout, stderr) = io.run_argv(&["sh", "-c", &command]).await?;
    if code != 0 {
        bail!("write remote file `{path}` failed: {}", stderr.trim());
    }
    Ok(())
}

async fn restart_target<I: HostIo + 'static>(
    io: Arc<I>,
    restart_model: Option<&RestartModel>,
) -> Result<RestartSelection> {
    let selection = match restart_model {
        Some(RestartModel::SystemService { service }) => {
            restart(io, Some(service.clone()), Some(ServiceScope::System)).await?;
            RestartSelection::SystemService
        }
        Some(RestartModel::UserService { service }) => {
            restart(io, Some(service.clone()), Some(ServiceScope::User)).await?;
            RestartSelection::UserService
        }
        Some(RestartModel::WrapperCommand { command }) => {
            run_wrapper_restart(io, command).await?;
            RestartSelection::WrapperCommand
        }
        None => {
            restart(io, None, None).await?;
            RestartSelection::WrapperCommand
        }
    };
    Ok(selection)
}

async fn restart_local_target(restart_model: Option<&RestartModel>) -> Result<RestartSelection> {
    let selection = match restart_model {
        Some(RestartModel::SystemService { service }) => {
            run_local_command(["systemctl", "restart", service.as_str()]).await?;
            run_local_command(["systemctl", "is-active", "--wait", service.as_str()]).await?;
            RestartSelection::SystemService
        }
        Some(RestartModel::UserService { service }) => {
            run_local_command(["systemctl", "--user", "restart", service.as_str()]).await?;
            run_local_command(["systemctl", "--user", "is-active", "--wait", service.as_str()])
                .await?;
            RestartSelection::UserService
        }
        Some(RestartModel::WrapperCommand { command }) => {
            run_local_command_vec(command).await?;
            RestartSelection::WrapperCommand
        }
        None => RestartSelection::WrapperCommand,
    };
    Ok(selection)
}

async fn run_wrapper_restart<I: HostIo + 'static>(io: Arc<I>, command: &[String]) -> Result<()> {
    if command.is_empty() {
        bail!("wrapper restart command must not be empty");
    }
    let argv: Vec<&str> = command.iter().map(String::as_str).collect();
    let (code, _stdout, stderr) = io.run_argv(&argv).await?;
    if code != 0 {
        bail!("wrapper restart failed: {}", stderr.trim());
    }
    Ok(())
}

async fn install_local_artifact(source: &Path, target: &Path) -> Result<()> {
    validate_remote_path(&target.display().to_string())
        .context("validate local controller install path")?;
    if let Some(parent) = target.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("create {}", parent.display()))?;
    }

    let staged = target.with_extension("new");
    let backup = target.with_extension(format!(
        "bak.{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs())
            .unwrap_or_default()
    ));

    tokio::fs::copy(source, &staged)
        .await
        .with_context(|| format!("copy {} -> {}", source.display(), staged.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        tokio::fs::set_permissions(&staged, std::fs::Permissions::from_mode(0o755))
            .await
            .with_context(|| format!("chmod {}", staged.display()))?;
    }

    if tokio::fs::try_exists(target).await.unwrap_or(false) {
        tokio::fs::rename(target, &backup)
            .await
            .with_context(|| format!("backup {} -> {}", target.display(), backup.display()))?;
    }
    tokio::fs::rename(&staged, target)
        .await
        .with_context(|| format!("install {} -> {}", staged.display(), target.display()))?;
    Ok(())
}

async fn verify_local_health() -> Result<()> {
    let mut stream = tokio::net::TcpStream::connect("127.0.0.1:8765")
        .await
        .context("connect to local controller health endpoint")?;
    stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
        .await
        .context("write local controller health request")?;
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .await
        .context("read local controller health response")?;
    let response = String::from_utf8_lossy(&response);
    if !response.starts_with("HTTP/1.1 200") && !response.starts_with("HTTP/1.0 200") {
        bail!("local controller health check did not return 200");
    }
    Ok(())
}

async fn run_local_command<const N: usize>(argv: [&str; N]) -> Result<()> {
    let output = tokio::process::Command::new(argv[0])
        .args(&argv[1..])
        .output()
        .await
        .with_context(|| format!("spawn {}", argv[0]))?;
    if !output.status.success() {
        bail!(
            "{} failed: {}",
            argv[0],
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

async fn run_local_command_vec(command: &[String]) -> Result<()> {
    if command.is_empty() {
        bail!("wrapper restart command must not be empty");
    }
    let output = tokio::process::Command::new(&command[0])
        .args(&command[1..])
        .output()
        .await
        .with_context(|| format!("spawn {}", command[0]))?;
    if !output.status.success() {
        bail!(
            "{} failed: {}",
            command[0],
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

fn current_home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn normalize_host_identifier(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_end_matches('.');
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_ascii_lowercase())
    }
}

fn hosts_match(left: &str, right: &str) -> bool {
    let Some(left) = normalize_host_identifier(left) else {
        return false;
    };
    let Some(right) = normalize_host_identifier(right) else {
        return false;
    };
    left == right
        || left.split('.').next().unwrap_or(&left) == right.split('.').next().unwrap_or(&right)
}

fn ssh_target_matches_local(target: &SshHostTarget, local_host: &str, controller_host: &str) -> bool {
    hosts_match(&target.alias, local_host)
        || target
            .hostname
            .as_deref()
            .map(|hostname| hosts_match(hostname, local_host))
            .unwrap_or(false)
        || hosts_match(&target.alias, controller_host)
        || target
            .hostname
            .as_deref()
            .map(|hostname| hosts_match(hostname, controller_host))
            .unwrap_or(false)
}

fn failed_result(
    target: String,
    kind: UpdateTargetKind,
    node_id: Option<String>,
    skipped_transfer: bool,
    failed_stage: String,
    stages_ms: BTreeMap<String, u128>,
    error: String,
) -> UpdateTargetResult {
    UpdateTargetResult {
        target,
        kind,
        node_id,
        connected: Some(false),
        controller_health_ok: Some(false),
        skipped_transfer,
        ok: false,
        failed_stage: Some(failed_stage),
        stages_ms,
        error: Some(error),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::dispatch::deploy::runner::test_support::{RecordingIo, RunResp};

    #[test]
    fn resolve_targets_adds_local_controller_last_for_all() {
        let config = LabConfig {
            node: Some(crate::config::NodePreferences {
                controller: Some("controller".into()),
                ..Default::default()
            }),
            deploy: Some(DeployPreferences::default()),
            ..LabConfig::default()
        };

        let resolved = resolve_targets(&config, "controller", "controller", Vec::new(), true)
            .expect("resolve");

        assert!(resolved.local_controller.is_some());
    }

    #[tokio::test]
    async fn normalize_remote_runtime_removes_legacy_files_and_writes_node_controller() {
        let io = Arc::new(RecordingIo::new());
        io.push_run(RunResp::ok("/home/lab"));
        io.push_run(RunResp::ok(""));
        io.push_run(RunResp::ok(""));

        normalize_remote_runtime(io.clone(), "mini1", "controller")
            .await
            .expect("normalize");

        let ops = io.ops();
        assert!(ops.iter().any(|op| op.contains(".lab/config.toml")));
        assert!(ops.iter().any(|op| op.contains("device-enrollments.json")));
    }

    #[tokio::test]
    async fn wrapper_restart_uses_raw_argv() {
        let io = Arc::new(RecordingIo::new());
        io.push_run(RunResp::ok(""));
        run_wrapper_restart(io.clone(), &["echo".into(), "restart".into()])
            .await
            .expect("restart");
        assert!(io.ops().iter().any(|op| op == "run:echo,restart"));
    }
}
