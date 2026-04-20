use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};

use super::catalog::ACTIONS;
use super::client::require_gateway_manager;
use super::manager::GatewayManager;
use super::params::{
    GatewayAddParams, GatewayNameParams, GatewayStatusParams, GatewayTestParams,
    GatewayUpdateParams, ServiceConfigGetParams, ServiceConfigSetParams,
    VirtualServerMcpPolicyParams, VirtualServerNameParams, VirtualServerSurfaceParams,
};
use super::types::ServiceActionView;

fn parse_params<T: DeserializeOwned>(params_value: Value) -> Result<T, ToolError> {
    serde_json::from_value(params_value).map_err(|e| ToolError::InvalidParam {
        message: format!("invalid gateway params: {e}"),
        param: "params".to_string(),
    })
}

pub async fn dispatch_with_manager(
    manager: &GatewayManager,
    action: &str,
    params_value: Value,
) -> Result<Value, ToolError> {
    match action {
        "help" => Ok(help_payload("gateway", ACTIONS)),
        "schema" => {
            let action_name = require_str(&params_value, "action")?;
            action_schema(ACTIONS, action_name)
        }
        "gateway.list" => to_json(manager.list().await?),
        "gateway.server.get" => {
            let params: VirtualServerNameParams = parse_params(params_value)?;
            to_json(manager.get_server(&params.id).await?)
        }
        "gateway.supported_services" => to_json(super::service_catalog::supported_services()),
        "gateway.virtual_server.enable" => {
            let params: VirtualServerNameParams = parse_params(params_value)?;
            to_json(manager.enable_virtual_server(&params.id).await?)
        }
        "gateway.virtual_server.disable" => {
            let params: VirtualServerNameParams = parse_params(params_value)?;
            to_json(manager.disable_virtual_server(&params.id).await?)
        }
        "gateway.virtual_server.set_surface" => {
            let params: VirtualServerSurfaceParams = parse_params(params_value)?;
            to_json(
                manager
                    .set_virtual_server_surface(&params.id, &params.surface, params.enabled)
                    .await?,
            )
        }
        "gateway.virtual_server.get_mcp_policy" => {
            let params: VirtualServerNameParams = parse_params(params_value)?;
            to_json(manager.get_virtual_server_mcp_policy(&params.id).await?)
        }
        "gateway.virtual_server.set_mcp_policy" => {
            let params: VirtualServerMcpPolicyParams = parse_params(params_value)?;
            let service = manager.service_for_virtual_server_id(&params.id).await?;
            let valid_actions = compiled_service_actions(&service)?;
            for action in &params.allowed_actions {
                if !valid_actions
                    .iter()
                    .any(|candidate| candidate.name == action.as_str())
                {
                    return Err(ToolError::InvalidParam {
                        message: format!(
                            "action `{action}` is not valid for service `{}`",
                            service
                        ),
                        param: "allowed_actions".to_string(),
                    });
                }
            }
            to_json(
                manager
                    .set_virtual_server_mcp_policy(&params.id, &params.allowed_actions)
                    .await?,
            )
        }
        "gateway.service_config.get" => {
            let params: ServiceConfigGetParams = parse_params(params_value)?;
            to_json(manager.get_service_config(&params.service).await?)
        }
        "gateway.service_config.set" => {
            let params: ServiceConfigSetParams = parse_params(params_value)?;
            to_json(
                manager
                    .set_service_config(&params.service, &params.values)
                    .await?,
            )
        }
        "gateway.service_actions" => {
            let params: ServiceConfigGetParams = parse_params(params_value)?;
            to_json(compiled_service_actions(&params.service)?)
        }
        "gateway.get" => {
            let params: GatewayNameParams = parse_params(params_value)?;
            to_json(manager.get(&params.name).await?)
        }
        "gateway.test" => {
            let params: GatewayTestParams = parse_params(params_value)?;
            match (params.name.as_deref(), params.spec.as_ref()) {
                (Some(name), None) => to_json(manager.test(Err(name)).await?),
                (None, Some(spec)) => to_json(manager.test(Ok(spec)).await?),
                (Some(_), Some(_)) => Err(ToolError::InvalidParam {
                    message: "gateway.test accepts either `name` or `spec`, not both".to_string(),
                    param: "name".to_string(),
                }),
                (None, None) => Err(ToolError::MissingParam {
                    message: "gateway.test requires either `name` or `spec`".to_string(),
                    param: "name".to_string(),
                }),
            }
        }
        "gateway.add" => {
            let params: GatewayAddParams = parse_params(params_value)?;
            to_json(manager.add(params.spec, params.bearer_token_value).await?)
        }
        "gateway.update" => {
            let params: GatewayUpdateParams = parse_params(params_value)?;
            to_json(
                manager
                    .update(&params.name, params.patch, params.bearer_token_value)
                    .await?,
            )
        }
        "gateway.remove" => {
            let params: GatewayNameParams = parse_params(params_value)?;
            to_json(manager.remove(&params.name).await?)
        }
        "gateway.reload" => to_json(manager.reload().await?),
        "gateway.status" => {
            let params: GatewayStatusParams = parse_params(params_value)?;
            to_json(manager.status(params.name.as_deref()).await?)
        }
        "gateway.discovered_tools" => {
            let params: GatewayNameParams = parse_params(params_value)?;
            to_json(manager.discovered_tools(&params.name).await?)
        }
        "gateway.discovered_resources" => {
            let params: GatewayNameParams = parse_params(params_value)?;
            to_json(manager.discovered_resources(&params.name).await?)
        }
        "gateway.discovered_prompts" => {
            let params: GatewayNameParams = parse_params(params_value)?;
            to_json(manager.discovered_prompts(&params.name).await?)
        }
        unknown => Err(ToolError::UnknownAction {
            message: format!("unknown action '{unknown}'"),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
            hint: None,
        }),
    }
}

fn compiled_service_actions(service: &str) -> Result<Vec<ServiceActionView>, ToolError> {
    let registry = crate::registry::build_default_registry();
    let entry = registry
        .service(service)
        .ok_or_else(|| ToolError::InvalidParam {
            message: format!("unknown service `{service}`"),
            param: "service".to_string(),
        })?;

    Ok(entry
        .actions
        .iter()
        .map(|action| ServiceActionView {
            name: action.name.to_string(),
            description: action.description.to_string(),
            destructive: action.destructive,
        })
        .collect())
}

pub async fn dispatch(action: &str, params_value: Value) -> Result<Value, ToolError> {
    let manager = require_gateway_manager()?;
    dispatch_with_manager(&manager, action, params_value).await
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::config::UpstreamConfig;

    use super::super::manager::GatewayRuntimeHandle;
    use super::*;

    #[test]
    fn gateway_actions_include_management_surface() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"gateway.list"));
        assert!(names.contains(&"gateway.server.get"));
        assert!(names.contains(&"gateway.supported_services"));
        assert!(names.contains(&"gateway.virtual_server.enable"));
        assert!(names.contains(&"gateway.virtual_server.disable"));
        assert!(names.contains(&"gateway.virtual_server.set_surface"));
        assert!(names.contains(&"gateway.virtual_server.get_mcp_policy"));
        assert!(names.contains(&"gateway.virtual_server.set_mcp_policy"));
        assert!(names.contains(&"gateway.service_config.get"));
        assert!(names.contains(&"gateway.service_config.set"));
        assert!(names.contains(&"gateway.service_actions"));
        assert!(names.contains(&"gateway.get"));
        assert!(names.contains(&"gateway.test"));
        assert!(names.contains(&"gateway.add"));
        assert!(names.contains(&"gateway.update"));
        assert!(names.contains(&"gateway.remove"));
        assert!(names.contains(&"gateway.reload"));
        assert!(names.contains(&"gateway.status"));
        assert!(names.contains(&"gateway.discovered_tools"));
        assert!(names.contains(&"gateway.discovered_resources"));
        assert!(names.contains(&"gateway.discovered_prompts"));

        for name in [
            "gateway.add",
            "gateway.update",
            "gateway.remove",
            "gateway.reload",
        ] {
            let spec = ACTIONS
                .iter()
                .find(|spec| spec.name == name)
                .expect("action");
            assert!(spec.destructive, "{name} must be destructive");
        }
    }

    fn test_manager() -> GatewayManager {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        GatewayManager::new(path, GatewayRuntimeHandle::default())
    }

    #[tokio::test]
    async fn gateway_list_returns_array() {
        let manager = test_manager();
        manager
            .replace_config_for_tests(vec![UpstreamConfig {
                name: "fixture-http".to_string(),
                url: Some("http://127.0.0.1:9001".to_string()),
                bearer_token_env: Some("FIXTURE_HTTP_TOKEN".to_string()),
                command: None,
                args: Vec::new(),
                proxy_resources: false,
                expose_tools: None,
                oauth: None,
            }])
            .await;

        let value = dispatch_with_manager(&manager, "gateway.list", json!({}))
            .await
            .expect("list");

        assert!(value.is_array());
        assert_eq!(value.as_array().expect("array").len(), 1);
        let row = &value.as_array().expect("array")[0];
        assert_eq!(row["discovered_tool_count"], 0);
        assert_eq!(row["exposed_tool_count"], 0);
        assert_eq!(row["discovered_resource_count"], 0);
        assert_eq!(row["exposed_resource_count"], 0);
        assert_eq!(row["discovered_prompt_count"], 0);
        assert_eq!(row["exposed_prompt_count"], 0);
    }

    #[tokio::test]
    async fn gateway_server_get_returns_custom_gateway_row() {
        let manager = test_manager();
        manager
            .replace_config_for_tests(vec![UpstreamConfig {
                name: "fixture-http".to_string(),
                url: Some("http://127.0.0.1:9001".to_string()),
                bearer_token_env: Some("FIXTURE_HTTP_TOKEN".to_string()),
                command: None,
                args: Vec::new(),
                proxy_resources: false,
                expose_tools: None,
                oauth: None,
            }])
            .await;

        let value =
            dispatch_with_manager(&manager, "gateway.server.get", json!({"id":"fixture-http"}))
                .await
                .expect("server get");

        assert_eq!(value["id"], "fixture-http");
        assert_eq!(value["source"], "custom_gateway");
    }

    #[tokio::test]
    async fn gateway_list_surfaces_cached_custom_gateway_summary_counts() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        let runtime = GatewayRuntimeHandle::default();
        let manager = GatewayManager::new(path, runtime.clone());

        manager
            .replace_config_for_tests(vec![UpstreamConfig {
                name: "noxa".to_string(),
                url: None,
                bearer_token_env: None,
                command: Some("noxa".to_string()),
                args: vec!["mcp".to_string()],
                proxy_resources: true,
                expose_tools: Some(vec!["scrape".to_string()]),
                oauth: None,
            }])
            .await;

        let pool = crate::dispatch::upstream::pool::UpstreamPool::new();
        let upstream_name: std::sync::Arc<str> = std::sync::Arc::from("noxa");
        let mut tools = std::collections::HashMap::new();
        for name in ["scrape", "crawl"] {
            let schema = std::sync::Arc::new(serde_json::Map::new());
            let tool = rmcp::model::Tool::new(name, format!("{name} description"), schema);
            tools.insert(
                name.to_string(),
                crate::dispatch::upstream::types::UpstreamTool {
                    tool,
                    input_schema: None,
                    upstream_name: std::sync::Arc::clone(&upstream_name),
                },
            );
        }
        pool.insert_entry_for_tests(
            "noxa",
            crate::dispatch::upstream::types::UpstreamEntry {
                name: std::sync::Arc::clone(&upstream_name),
                tools,
                exposure_policy:
                    crate::dispatch::upstream::types::ToolExposurePolicy::from_patterns(vec![
                        "scrape".to_string(),
                    ])
                    .expect("policy"),
                prompt_count: 3,
                resource_count: 4,
                prompt_names: Vec::new(),
                resource_uris: Vec::new(),
                tool_health: crate::dispatch::upstream::types::UpstreamHealth::Healthy,
                prompt_health: crate::dispatch::upstream::types::UpstreamHealth::Healthy,
                resource_health: crate::dispatch::upstream::types::UpstreamHealth::Healthy,
                tool_unhealthy_since: None,
                prompt_unhealthy_since: None,
                resource_unhealthy_since: None,
                tool_last_error: None,
                prompt_last_error: None,
                resource_last_error: None,
            },
        )
        .await;
        runtime.swap(Some(std::sync::Arc::new(pool))).await;

        let value = dispatch_with_manager(&manager, "gateway.list", json!({}))
            .await
            .expect("list");
        let row = value
            .as_array()
            .expect("array")
            .iter()
            .find(|item| item["id"] == "noxa")
            .expect("noxa row");

        assert_eq!(row["discovered_tool_count"], 2);
        assert_eq!(row["exposed_tool_count"], 1);
        assert_eq!(row["discovered_resource_count"], 4);
        assert_eq!(row["exposed_resource_count"], 4);
        assert_eq!(row["discovered_prompt_count"], 3);
        assert_eq!(row["exposed_prompt_count"], 3);
    }

    #[tokio::test]
    async fn virtual_server_policy_validation_uses_service_name() {
        let manager = test_manager();
        manager
            .seed_config(crate::config::LabConfig {
                virtual_servers: vec![crate::config::VirtualServerConfig {
                    id: "plex-primary".to_string(),
                    service: "plex".to_string(),
                    enabled: true,
                    surfaces: crate::config::VirtualServerSurfacesConfig::default(),
                    mcp_policy: None,
                }],
                ..crate::config::LabConfig::default()
            })
            .await;

        let value = dispatch_with_manager(
            &manager,
            "gateway.virtual_server.set_mcp_policy",
            json!({"id":"plex-primary","allowed_actions":["server.info"]}),
        )
        .await
        .expect("set policy");

        assert_eq!(value["allowed_actions"][0], "server.info");
    }

    #[test]
    fn supported_services_lists_metadata_backed_lab_gateways() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"gateway.supported_services"));
    }

    #[tokio::test]
    async fn supported_services_payload_includes_plex_when_feature_enabled() {
        let manager = test_manager();
        let value = dispatch_with_manager(&manager, "gateway.supported_services", json!({}))
            .await
            .expect("supported services");

        let services = value.as_array().expect("array");
        #[cfg(feature = "plex")]
        assert!(services.iter().any(|service| service["key"] == "plex"));
    }

    #[tokio::test]
    async fn enabling_virtual_server_marks_existing_server_row_enabled() {
        let manager = test_manager();
        manager
            .seed_config(crate::config::LabConfig {
                virtual_servers: vec![crate::config::VirtualServerConfig {
                    id: "plex".to_string(),
                    service: "plex".to_string(),
                    enabled: false,
                    surfaces: crate::config::VirtualServerSurfacesConfig::default(),
                    mcp_policy: None,
                }],
                ..crate::config::LabConfig::default()
            })
            .await;

        let value = dispatch_with_manager(
            &manager,
            "gateway.virtual_server.enable",
            json!({"id": "plex"}),
        )
        .await
        .expect("enable");

        assert_eq!(value["id"], "plex");
        assert_eq!(value["enabled"], true);
    }

    #[tokio::test]
    async fn enabling_virtual_server_creates_missing_service_row() {
        let manager = test_manager();

        dispatch_with_manager(
            &manager,
            "gateway.service_config.set",
            json!({
                "service": "plex",
                "values": {
                    "PLEX_URL": "http://127.0.0.1:32400",
                    "PLEX_TOKEN": "token"
                }
            }),
        )
        .await
        .expect("set service config");

        let value = dispatch_with_manager(
            &manager,
            "gateway.virtual_server.enable",
            json!({"id": "plex"}),
        )
        .await
        .expect("enable missing virtual server");

        assert_eq!(value["id"], "plex");
        assert_eq!(value["source"], "lab_service");
        assert_eq!(value["enabled"], true);
        assert_eq!(value["surfaces"]["mcp"]["enabled"], true);
    }

    #[tokio::test]
    async fn disabling_virtual_server_keeps_server_row_visible_but_disabled() {
        let manager = test_manager();
        manager
            .seed_config(crate::config::LabConfig {
                virtual_servers: vec![crate::config::VirtualServerConfig {
                    id: "plex".to_string(),
                    service: "plex".to_string(),
                    enabled: true,
                    surfaces: crate::config::VirtualServerSurfacesConfig::default(),
                    mcp_policy: None,
                }],
                ..crate::config::LabConfig::default()
            })
            .await;

        let value = dispatch_with_manager(
            &manager,
            "gateway.virtual_server.disable",
            json!({"id": "plex"}),
        )
        .await
        .expect("disable");

        assert_eq!(value["id"], "plex");
        assert_eq!(value["enabled"], false);

        let list = dispatch_with_manager(&manager, "gateway.list", json!({}))
            .await
            .expect("list after disable");
        assert!(
            list.as_array()
                .expect("array")
                .iter()
                .any(|server| server["id"] == "plex" && server["enabled"] == false)
        );
    }

    #[tokio::test]
    async fn setting_service_config_writes_canonical_env_backed_fields() {
        let manager = test_manager();

        let value = dispatch_with_manager(
            &manager,
            "gateway.service_config.set",
            json!({
                "service": "plex",
                "values": {
                    "PLEX_URL": "http://127.0.0.1:32400",
                    "PLEX_TOKEN": "token"
                }
            }),
        )
        .await
        .expect("set service config");

        assert_eq!(value["service"], "plex");
        assert_eq!(value["configured"], true);
        assert!(
            value["fields"]
                .as_array()
                .expect("fields")
                .iter()
                .any(|field| field["name"] == "PLEX_URL" && field["present"] == true)
        );
        assert!(
            value["fields"]
                .as_array()
                .expect("fields")
                .iter()
                .any(|field| field["name"] == "PLEX_TOKEN" && field["present"] == true)
        );
    }

    #[tokio::test]
    async fn configured_but_disabled_service_can_be_read_back_for_editing() {
        let manager = test_manager();
        manager
            .seed_config(crate::config::LabConfig {
                virtual_servers: vec![crate::config::VirtualServerConfig {
                    id: "plex".to_string(),
                    service: "plex".to_string(),
                    enabled: false,
                    surfaces: crate::config::VirtualServerSurfacesConfig::default(),
                    mcp_policy: None,
                }],
                ..crate::config::LabConfig::default()
            })
            .await;

        dispatch_with_manager(
            &manager,
            "gateway.service_config.set",
            json!({
                "service": "plex",
                "values": {
                    "PLEX_URL": "http://127.0.0.1:32400",
                    "PLEX_TOKEN": "token"
                }
            }),
        )
        .await
        .expect("set service config");

        let value = dispatch_with_manager(
            &manager,
            "gateway.service_config.get",
            json!({"service": "plex"}),
        )
        .await
        .expect("get service config");

        assert_eq!(value["service"], "plex");
        assert_eq!(value["configured"], true);
        assert!(
            value["fields"]
                .as_array()
                .expect("fields")
                .iter()
                .any(|field| field["name"] == "PLEX_URL"
                    && field["value_preview"] == "http://127.0.0.1:32400")
        );
        assert!(
            value["fields"]
                .as_array()
                .expect("fields")
                .iter()
                .any(|field| field["name"] == "PLEX_TOKEN" && field["secret"] == true)
        );
    }

    #[tokio::test]
    async fn setting_virtual_server_surface_updates_visible_server_row() {
        let manager = test_manager();
        manager
            .seed_config(crate::config::LabConfig {
                virtual_servers: vec![crate::config::VirtualServerConfig {
                    id: "plex".to_string(),
                    service: "plex".to_string(),
                    enabled: true,
                    surfaces: crate::config::VirtualServerSurfacesConfig {
                        mcp: true,
                        ..crate::config::VirtualServerSurfacesConfig::default()
                    },
                    mcp_policy: None,
                }],
                ..crate::config::LabConfig::default()
            })
            .await;

        let value = dispatch_with_manager(
            &manager,
            "gateway.virtual_server.set_surface",
            json!({"id": "plex", "surface": "api", "enabled": true}),
        )
        .await
        .expect("set surface");

        assert_eq!(value["id"], "plex");
        assert_eq!(value["surfaces"]["api"]["enabled"], true);
    }

    #[tokio::test]
    async fn setting_virtual_server_mcp_policy_persists_allowed_actions() {
        let manager = test_manager();
        manager
            .seed_config(crate::config::LabConfig {
                virtual_servers: vec![crate::config::VirtualServerConfig {
                    id: "plex".to_string(),
                    service: "plex".to_string(),
                    enabled: true,
                    surfaces: crate::config::VirtualServerSurfacesConfig {
                        cli: false,
                        api: false,
                        mcp: true,
                        webui: false,
                    },
                    mcp_policy: None,
                }],
                ..crate::config::LabConfig::default()
            })
            .await;

        let value = dispatch_with_manager(
            &manager,
            "gateway.virtual_server.set_mcp_policy",
            json!({"id": "plex", "allowed_actions": ["server.info"]}),
        )
        .await
        .expect("set mcp policy");

        assert_eq!(value["allowed_actions"], json!(["server.info"]));

        let reloaded = dispatch_with_manager(
            &manager,
            "gateway.virtual_server.get_mcp_policy",
            json!({"id": "plex"}),
        )
        .await
        .expect("get mcp policy");

        assert_eq!(reloaded["allowed_actions"], json!(["server.info"]));
    }

    #[tokio::test]
    async fn service_actions_returns_compiled_action_catalog() {
        let manager = test_manager();
        let value = dispatch_with_manager(
            &manager,
            "gateway.service_actions",
            json!({"service": "plex"}),
        )
        .await
        .expect("service actions");

        let actions = value.as_array().expect("array");
        assert!(actions.iter().any(|action| action["name"] == "server.info"));
    }

    #[tokio::test]
    async fn gateway_get_rejects_missing_name() {
        let manager = test_manager();
        let err = dispatch_with_manager(&manager, "gateway.get", json!({}))
            .await
            .expect_err("missing name should fail");

        assert_eq!(err.kind(), "invalid_param");
    }

    #[tokio::test]
    async fn gateway_test_accepts_name_or_spec() {
        let manager = test_manager();
        manager
            .replace_config_for_tests(vec![UpstreamConfig {
                name: "fixture-http".to_string(),
                url: Some("http://127.0.0.1:9001".to_string()),
                bearer_token_env: None,
                command: None,
                args: Vec::new(),
                proxy_resources: false,
                expose_tools: None,
                oauth: None,
            }])
            .await;

        let named =
            dispatch_with_manager(&manager, "gateway.test", json!({"name": "fixture-http"}))
                .await
                .expect("named test");
        let proposed = dispatch_with_manager(
            &manager,
            "gateway.test",
            json!({"spec": {
                "name": "fixture-stdio",
                "command": "echo",
                "args": ["hello"]
            }}),
        )
        .await
        .expect("spec test");

        assert_eq!(named["name"], "fixture-http");
        assert_eq!(proposed["name"], "fixture-stdio");
    }

    #[tokio::test]
    async fn gateway_mutations_call_manager_methods() {
        let manager = test_manager();

        let added = dispatch_with_manager(
            &manager,
            "gateway.add",
            json!({"spec": {
                "name": "fixture-http",
                "url": "https://fixture.example.com/mcp",
                "bearer_token_env": "FIXTURE_HTTP_TOKEN"
            }}),
        )
        .await
        .expect("add");
        assert_eq!(added["config"]["name"], "fixture-http");
        assert_eq!(added["config"]["bearer_token_env"], "FIXTURE_HTTP_TOKEN");

        let updated = dispatch_with_manager(
            &manager,
            "gateway.update",
            json!({"name": "fixture-http", "patch": {"proxy_resources": true}}),
        )
        .await
        .expect("update");
        assert_eq!(updated["config"]["proxy_resources"], true);

        let status = dispatch_with_manager(&manager, "gateway.status", json!({}))
            .await
            .expect("status");
        assert!(status.is_array());

        let removed =
            dispatch_with_manager(&manager, "gateway.remove", json!({"name": "fixture-http"}))
                .await
                .expect("remove");
        assert_eq!(removed["config"]["name"], "fixture-http");

        let reloaded = dispatch_with_manager(&manager, "gateway.reload", json!({}))
            .await
            .expect("reload");
        assert!(reloaded.get("tools_changed").is_some());
    }

    #[tokio::test]
    async fn invalid_gateway_specs_return_validation_errors() {
        let manager = test_manager();

        let invalid_url = dispatch_with_manager(
            &manager,
            "gateway.add",
            json!({"spec": {"name": "bad", "url": "ftp://example.com"}}),
        )
        .await
        .expect_err("invalid scheme");
        assert_eq!(invalid_url.kind(), "invalid_param");

        let invalid_transport = dispatch_with_manager(
            &manager,
            "gateway.add",
            json!({"spec": {"name": "bad", "url": "http://127.0.0.1:9001", "command": "node"}}),
        )
        .await
        .expect_err("invalid transport");
        assert_eq!(invalid_transport.kind(), "invalid_param");
    }

    #[tokio::test]
    async fn only_reload_promises_to_pick_up_changed_bearer_token_env_vars() {
        let manager = test_manager();
        manager
            .replace_config_for_tests(vec![UpstreamConfig {
                name: "fixture-http".to_string(),
                url: Some("http://127.0.0.1:9001".to_string()),
                bearer_token_env: Some("FIXTURE_HTTP_TOKEN".to_string()),
                command: None,
                args: Vec::new(),
                proxy_resources: false,
                expose_tools: None,
                oauth: None,
            }])
            .await;

        let status = dispatch_with_manager(&manager, "gateway.status", json!({}))
            .await
            .expect("status");
        assert!(status.is_array());

        let help = dispatch_with_manager(&manager, "help", json!({}))
            .await
            .expect("help");
        assert_eq!(help["service"], "gateway");
        assert!(
            help.to_string().contains("gateway.reload"),
            "reload should remain the explicit env-refresh action"
        );
    }
}
