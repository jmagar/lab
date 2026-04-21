use lab_apis::mcpregistry::McpRegistryClient;
use lab_apis::mcpregistry::types::ServerJSON;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, to_json};
use crate::dispatch::mcpregistry::{catalog::ACTIONS, client, params};

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

            let url = server
                .remotes
                .iter()
                .find_map(|r| r.url.as_deref())
                .ok_or_else(|| ToolError::Sdk {
                    sdk_kind: "no_remote_transport".to_string(),
                    message: format!(
                        "server '{name}' has no remote transport URLs — stdio-only servers \
                         cannot be added as gateway upstreams"
                    ),
                })?
                .to_string();

            // SSRF validation (synchronous DNS) — must run in spawn_blocking.
            let url_for_check = url.clone();
            tokio::task::spawn_blocking(move || params::validate_registry_url(&url_for_check))
                .await
                .map_err(|e| {
                    ToolError::internal_message(format!("SSRF validation task panicked: {e}"))
                })??;

            // Probe for OAuth support and capture the discovered OAuth config if available.
            // Non-fatal: if there is no gateway manager or the probe fails for any reason,
            // install proceeds without OAuth configuration.
            let discovered_oauth: Option<Value> =
                if let Some(manager) = crate::dispatch::gateway::current_gateway_manager() {
                    match manager.probe_upstream_oauth(&url).await {
                        Ok(probe) if probe.oauth_discovered => {
                            // The probe registered a transient manager in the DashMap.
                            // Retrieve its full UpstreamOauthConfig so we can embed it in the
                            // gateway spec — that way the installed gateway already has OAuth
                            // and does not require a separate `gateway.oauth.probe` call.
                            let oauth_cfg = manager
                                .upstream_oauth_manager(&probe.upstream)
                                .and_then(|m| {
                                    serde_json::to_value(m.upstream_config().oauth.clone()).ok()
                                });
                            oauth_cfg
                        }
                        Ok(_) | Err(_) => None,
                    }
                } else {
                    None
                };

            let gateway_name = params_value["gateway_name"]
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| name.rsplit('/').next().unwrap_or(&name).to_string());

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

            // Delegate to gateway.add — pass confirm:true since the caller already confirmed at
            // the server.install level (destructive:true is checked by handle_action before we
            // reach this arm).
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
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action '{unknown}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
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
