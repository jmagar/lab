use lab_apis::mcpregistry::McpRegistryClient;
use lab_apis::mcpregistry::types::ServerJSON;
use serde_json::Value;

use crate::config;
use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, to_json};
use crate::dispatch::mcpregistry::{
    LAB_REGISTRY_META_NAMESPACE, catalog::ACTIONS, client, params, sync,
};

/// Dispatch using a pre-built client (avoids per-request env reads and client construction).
pub async fn dispatch_with_client(
    client: &McpRegistryClient,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        "config" => Ok(serde_json::json!({ "url": client::resolved_url() })),
        "server.meta.get" => dispatch_local(action, params_value).await,
        "server.meta.set" => dispatch_local(action, params_value).await,
        "server.meta.delete" => dispatch_local(action, params_value).await,
        "server.list" => {
            let p = params::list_servers_params(&params_value)?;
            if let Some(param) = ["sort_by", "order"]
                .into_iter()
                .find(|name| params_value.get(*name).is_some())
            {
                return Err(ToolError::InvalidParam {
                    message: "sort_by/order are not supported on the paginated registry surface".to_string(),
                    param: param.to_string(),
                });
            }
            to_json(client.list_servers(p).await?)
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
                        sdk_kind: "invalid_param".to_string(),
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
                    sdk_kind: "invalid_param".to_string(),
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
            // Open the registry store from config path (no AppState available in MCP/CLI path).
            let db_path = config::registry_db_path();
            let store =
                crate::dispatch::mcpregistry::store::RegistryStore::open(&db_path)
                    .await
                    .map_err(|e| {
                        ToolError::internal_message(format!("registry store open: {e}"))
                    })?;

            let count = sync::perform_sync(&store, client, true, "manual").await?;
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
        sdk_kind: "invalid_param".to_string(),
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
    if matches!(action, "config" | "server.meta.get" | "server.meta.set" | "server.meta.delete") {
        return dispatch_local(action, params_value).await;
    }
    dispatch_with_client(&client::require_client()?, action, params_value).await
}

async fn dispatch_local(action: &str, params_value: Value) -> Result<Value, ToolError> {
    match action {
        "config" => Ok(serde_json::json!({ "url": client::resolved_url() })),
        "server.meta.get" => {
            let name = params::require_name(&params_value)?;
            let requested_version = params_value["version"].as_str().unwrap_or("latest");
            let store = crate::dispatch::mcpregistry::store::RegistryStore::open(
                &config::registry_db_path(),
            )
            .await?;
            let server = store
                .get_server(&name, requested_version)
                .await?
                .ok_or_else(|| ToolError::Sdk {
                    sdk_kind: "not_found".to_string(),
                    message: format!(
                        "server '{name}' version '{requested_version}' not found in local registry store"
                    ),
                })?;
            let resolved_version = server.server.version.clone();
            let metadata = store.get_local_metadata(&name, &resolved_version).await?;
            Ok(serde_json::json!({
                "name": name,
                "version": resolved_version,
                "namespace": LAB_REGISTRY_META_NAMESPACE,
                "metadata": metadata,
            }))
        }
        "server.meta.set" => {
            let name = params::require_name(&params_value)?;
            let requested_version = params_value["version"].as_str().unwrap_or("latest");
            let metadata = params_value
                .get("metadata")
                .cloned()
                .ok_or_else(|| ToolError::MissingParam {
                    message: "missing required parameter `metadata`".to_string(),
                    param: "metadata".to_string(),
                })?;
            let metadata = params::parse_lab_metadata(&metadata)?;
            let updated_by = params_value
                .get("updated_by")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("unknown");

            let store = crate::dispatch::mcpregistry::store::RegistryStore::open(
                &config::registry_db_path(),
            )
            .await?;
            let server = store
                .get_server(&name, requested_version)
                .await?
                .ok_or_else(|| ToolError::Sdk {
                    sdk_kind: "not_found".to_string(),
                    message: format!(
                        "server '{name}' version '{requested_version}' not found in local registry store"
                    ),
                })?;
            let resolved_version = server.server.version.clone();
            let metadata_value = serde_json::to_value(metadata)
                .map_err(|e| ToolError::internal_message(format!("serialize lab metadata: {e}")))?;
            store
                .set_local_metadata(&name, &resolved_version, &metadata_value, Some(updated_by))
                .await?;
            let current = store.get_local_metadata(&name, &resolved_version).await?;
            Ok(serde_json::json!({
                "name": name,
                "version": resolved_version,
                "namespace": LAB_REGISTRY_META_NAMESPACE,
                "metadata": current,
            }))
        }
        "server.meta.delete" => {
            let name = params::require_name(&params_value)?;
            let requested_version = params_value["version"].as_str().unwrap_or("latest");
            let store = crate::dispatch::mcpregistry::store::RegistryStore::open(
                &config::registry_db_path(),
            )
            .await?;
            let server = store
                .get_server(&name, requested_version)
                .await?
                .ok_or_else(|| ToolError::Sdk {
                    sdk_kind: "not_found".to_string(),
                    message: format!(
                        "server '{name}' version '{requested_version}' not found in local registry store"
                    ),
                })?;
            let resolved_version = server.server.version.clone();
            let deleted = store
                .delete_local_metadata(&name, &resolved_version)
                .await?;
            Ok(serde_json::json!({
                "name": name,
                "version": resolved_version,
                "namespace": LAB_REGISTRY_META_NAMESPACE,
                "deleted": deleted,
            }))
        }
        _ => Err(ToolError::UnknownAction {
            message: format!("unknown action '{action}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}
