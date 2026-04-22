use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use lab_apis::mcpregistry::McpRegistryClient;
use lab_apis::mcpregistry::types::ServerJSON;
use serde_json::Value;

use crate::config;
use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, to_json};
use crate::dispatch::mcpregistry::{catalog::ACTIONS, client, params};

// ── Sync rate-limit state ─────────────────────────────────────────────────────

/// Guards against concurrent syncs. Set to `true` while a sync is in progress.
/// Reset to `false` via `SyncGuard` RAII on completion or panic.
static SYNC_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// Tracks when the last successful sync completed.
static LAST_SYNC_AT: OnceLock<std::sync::Mutex<Option<std::time::Instant>>> = OnceLock::new();

/// RAII guard that resets `SYNC_IN_PROGRESS` to `false` on drop, even on panic.
struct SyncGuard;

impl Drop for SyncGuard {
    fn drop(&mut self) {
        SYNC_IN_PROGRESS.store(false, Ordering::Release);
    }
}

/// Dispatch using a pre-built client (avoids per-request env reads and client construction).
pub async fn dispatch_with_client(
    client: &McpRegistryClient,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        "config" => Ok(serde_json::json!({ "url": client::resolved_url() })),
        "server.list" => {
            let p = params::list_servers_params(&params_value)?;
            let sort = params::parse_sort_params(&params_value)?;
            let mut resp = client.list_servers(p).await?;
            if let Some(spec) = sort {
                sort_servers(&mut resp.servers, &spec);
            }
            to_json(resp)
        }
        "server.get" => {
            let name = params::require_name(&params_value)?;
            to_json(client.get_server(&name, "latest").await?)
        }
        "server.versions" => {
            let name = params::require_name(&params_value)?;
            to_json(client.list_versions(&name).await?)
        }
        "server.validate" => {
            let server_json: ServerJSON =
                serde_json::from_value(params_value["server_json"].clone()).map_err(|e| {
                    ToolError::Sdk {
                        sdk_kind: "invalid_params".to_string(),
                        message: format!("invalid server_json: {e}"),
                    }
                })?;
            to_json(client.validate(&server_json).await?)
        }
        "server.install" => {
            let name = params::require_name(&params_value)?;
            let version = params_value["version"].as_str().unwrap_or("latest");

            let server_resp = client.get_server(&name, version).await?;
            let server = &server_resp.server;

            let gateway_name = params_value["gateway_name"]
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| name.rsplit('/').next().unwrap_or(&name).to_string());

            // Choose transport: prefer the first HTTP remote; fall back to packages[0] stdio.
            let http_url = server.remotes.iter().find_map(|r| r.url.as_deref());

            let spec = if let Some(url) = http_url {
                install_http(url, &gateway_name, &params_value).await?
            } else if let Some(pkg) = server.packages.first() {
                install_stdio(pkg, &gateway_name, &params_value, &name)?
            } else {
                return Err(ToolError::Sdk {
                    sdk_kind: "no_transport".to_string(),
                    message: format!(
                        "server '{name}' has no remotes and no packages — cannot install"
                    ),
                });
            };

            // Delegate to gateway.add — confirm:true because the caller already confirmed at the
            // server.install level (destructive:true is enforced by handle_action upstream).
            crate::dispatch::gateway::dispatch(
                "gateway.add",
                serde_json::json!({ "spec": spec, "confirm": true }),
            )
            .await
        }
        "server.uninstall" => {
            let gateway_name = params_value["gateway_name"]
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| ToolError::MissingParam {
                    message: "missing required parameter `gateway_name`".to_string(),
                    param: "gateway_name".to_string(),
                })?
                .to_string();

            // Delegate to gateway.remove — pass confirm:true since the caller already confirmed
            // at the server.uninstall level (destructive:true is checked by handle_action).
            crate::dispatch::gateway::dispatch(
                "gateway.remove",
                serde_json::json!({ "name": gateway_name, "confirm": true }),
            )
            .await
        }
        "sync" => {
            // Gate 1: reject if a sync is already in progress.
            // AcqRel is sufficient — Acquire establishes happens-before with the prior Release
            // store in SyncGuard::drop; Relaxed on failure is safe (we just read, no ordering needed).
            if SYNC_IN_PROGRESS
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
                .is_err()
            {
                return Err(ToolError::Sdk {
                    sdk_kind: "rate_limited".to_string(),
                    message: "sync already in progress".to_string(),
                });
            }
            // RAII guard: resets SYNC_IN_PROGRESS to false on drop, including on panic.
            // Must be created before any early return so all exit paths reset the flag.
            let _guard = SyncGuard;

            // Gate 2: minimum 60s between syncs.
            let last = LAST_SYNC_AT.get_or_init(|| std::sync::Mutex::new(None));
            {
                let guard = last.lock().unwrap();
                if let Some(t) = *guard {
                    if t.elapsed() < std::time::Duration::from_secs(60) {
                        // _guard drops here, resetting SYNC_IN_PROGRESS.
                        return Err(ToolError::Sdk {
                            sdk_kind: "rate_limited".to_string(),
                            message: format!(
                                "sync rate-limited; next allowed in {}s",
                                60 - t.elapsed().as_secs()
                            ),
                        });
                    }
                }
            }

            // Open the registry store from config path (no AppState available in MCP/CLI path).
            let db_path = config::registry_db_path();
            let store =
                crate::dispatch::mcpregistry::store::RegistryStore::open(&db_path)
                    .await
                    .map_err(|e| {
                        ToolError::internal_message(format!("registry store open: {e}"))
                    })?;

            let count = store.sync_from_upstream(client).await.map_err(|e| {
                ToolError::internal_message(format!("sync failed: {e}"))
            })?;

            // Update last sync time only on success.
            *LAST_SYNC_AT
                .get_or_init(|| std::sync::Mutex::new(None))
                .lock()
                .unwrap() = Some(std::time::Instant::now());

            Ok(serde_json::json!({ "synced": count }))
        }
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action '{unknown}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

/// Build a gateway spec for an HTTP-transport server and return it as JSON.
///
/// Validates the URL against SSRF rules and probes for OAuth support.
async fn install_http(
    url: &str,
    gateway_name: &str,
    params_value: &Value,
) -> Result<Value, ToolError> {
    let url = url.to_string();

    // SSRF validation (synchronous DNS) — must run in spawn_blocking.
    let url_for_check = url.clone();
    tokio::task::spawn_blocking(move || params::validate_registry_url(&url_for_check))
        .await
        .map_err(|e| ToolError::internal_message(format!("SSRF validation task panicked: {e}")))?
        ?;

    // Probe for OAuth support — non-fatal, install proceeds without OAuth on failure.
    let discovered_oauth: Option<Value> =
        if let Some(manager) = crate::dispatch::gateway::current_gateway_manager() {
            match manager.probe_upstream_oauth(&url).await {
                Ok(probe) if probe.oauth_discovered => manager
                    .upstream_oauth_manager(&probe.upstream)
                    .and_then(|m| serde_json::to_value(m.upstream_config().oauth.clone()).ok()),
                Ok(_) | Err(_) => None,
            }
        } else {
            None
        };

    let bearer_token_env = params_value["bearer_token_env"].as_str();

    let mut spec = serde_json::json!({
        "name": gateway_name,
        "url": url,
        "bearer_token_env": bearer_token_env,
        "command": null,
        "args": [],
        "proxy_resources": false,
        "expose_tools": null,
    });

    if let Some(oauth) = discovered_oauth {
        spec["oauth"] = oauth;
    }

    Ok(spec)
}

/// Build a gateway spec for a stdio-transport server, validate security constraints,
/// and write any user-supplied env vars into `~/.lab/.env`.
fn install_stdio(
    pkg: &lab_apis::mcpregistry::types::Package,
    gateway_name: &str,
    params_value: &Value,
    server_name: &str,
) -> Result<Value, ToolError> {
    // runtimeHint is required for stdio packages.
    let hint = pkg.runtime_hint.as_deref().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "no_runtime_hint".to_string(),
        message: format!(
            "server '{server_name}' package has no runtimeHint — cannot build stdio command"
        ),
    })?;

    params::validate_runtime_hint(hint)?;

    // Build argv: runtimeArguments + identifier + packageArguments.
    let mut argv: Vec<String> = pkg
        .runtime_arguments
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    argv.push(pkg.identifier.clone());
    argv.extend(
        pkg.package_arguments
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string)),
    );

    params::validate_stdio_argv(&argv)?;

    // Resolve env vars: user-supplied values take precedence; defaults fill gaps;
    // required vars with no resolution produce an error.
    let user_env = params_value["env_values"].as_object();
    let mut resolved_env: Vec<(String, String)> = Vec::new();

    for ev in &pkg.environment_variables {
        params::validate_env_var_name(&ev.name)?;

        let user_val = user_env
            .and_then(|m| m.get(&ev.name))
            .and_then(Value::as_str);

        let resolved = if let Some(v) = user_val {
            params::validate_env_value(&ev.name, v)?;
            Some(v.to_string())
        } else {
            ev.default.clone()
        };

        match resolved {
            Some(v) => resolved_env.push((ev.name.clone(), v)),
            None if ev.is_required => {
                return Err(ToolError::MissingParam {
                    message: format!(
                        "env var '{}' is required by server '{server_name}' but was not supplied in \
                         `env_values` and has no default",
                        ev.name
                    ),
                    param: ev.name.clone(),
                });
            }
            None => {}
        }
    }

    // Write resolved env vars to ~/.lab/.env if there are any.
    if !resolved_env.is_empty() {
        let env_path = config::dotenv_path().ok_or_else(|| ToolError::Sdk {
            sdk_kind: "internal_error".to_string(),
            message: "cannot determine ~/.lab/.env path".to_string(),
        })?;
        config::backup_env(&env_path).map_err(|e| ToolError::Sdk {
            sdk_kind: "internal_error".to_string(),
            message: format!("failed to back up .env: {e}"),
        })?;
        let conflicts =
            config::write_env_pairs(&env_path, &resolved_env, false).map_err(|e| ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message: format!("failed to write env vars to .env: {e}"),
            })?;
        if !conflicts.is_empty() {
            // Surface conflicts as a warning embedded in the result rather than failing —
            // the gateway will still be registered; the user can --force if needed.
            tracing::warn!(
                service = "mcpregistry",
                action = "server.install",
                "env write conflicts (skipped): {}",
                conflicts.join("; ")
            );
        }
    }

    Ok(serde_json::json!({
        "name": gateway_name,
        "url": null,
        "command": hint,
        "args": argv,
        "proxy_resources": false,
        "expose_tools": null,
    }))
}

fn sort_key(s: &lab_apis::mcpregistry::types::ServerResponse, by: &params::SortBy) -> String {
    match by {
        params::SortBy::Name => s.server.name.clone(),
        params::SortBy::Published => s
            .meta
            .as_ref()
            .and_then(|m| m.official.as_ref())
            .map(|e| e.published_at.clone())
            .unwrap_or_default(),
        params::SortBy::Updated => s
            .meta
            .as_ref()
            .and_then(|m| m.official.as_ref())
            .map(|e| e.updated_at.as_deref().unwrap_or(&e.published_at).to_string())
            .unwrap_or_default(),
    }
}

fn sort_servers(
    servers: &mut Vec<lab_apis::mcpregistry::types::ServerResponse>,
    spec: &params::SortSpec,
) {
    servers.sort_by(|a, b| {
        let ka = sort_key(a, &spec.by);
        let kb = sort_key(b, &spec.by);
        if spec.desc { kb.cmp(&ka) } else { ka.cmp(&kb) }
    });
}

/// Dispatch one call against the `mcpregistry` service.
///
/// Returns a surface-neutral `Result<Value, ToolError>` consumed by MCP, CLI,
/// and API.
pub async fn dispatch(action: &str, params_value: Value) -> Result<Value, ToolError> {
    match action {
        "help" => return Ok(help_payload("mcpregistry", ACTIONS)),
        "schema" => {
            let action_name = crate::dispatch::helpers::require_str(&params_value, "action")?;
            return action_schema(ACTIONS, action_name);
        }
        _ if !ACTIONS.iter().any(|a| a.name == action) => {
            return Err(ToolError::UnknownAction {
                message: format!("unknown action '{action}'"),
                valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
                hint: None,
            });
        }
        _ => {}
    }
    dispatch_with_client(&client::require_client()?, action, params_value).await
}
