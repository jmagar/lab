use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::{action_schema, help_payload, require_str, to_json};

use super::catalog::ACTIONS;
use super::client::require_gateway_manager;
use super::manager::GatewayManager;
use super::params::{
    GatewayAddParams, GatewayNameParams, GatewayStatusParams, GatewayTestParams,
    GatewayUpdateParams, VirtualServerNameParams,
};

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
        "gateway.supported_services" => to_json(super::service_catalog::supported_services()),
        "gateway.virtual_server.enable" => {
            let params: VirtualServerNameParams = parse_params(params_value)?;
            to_json(manager.enable_virtual_server(&params.id).await?)
        }
        "gateway.virtual_server.disable" => {
            let params: VirtualServerNameParams = parse_params(params_value)?;
            to_json(manager.disable_virtual_server(&params.id).await?)
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
            to_json(manager.add(params.spec).await?)
        }
        "gateway.update" => {
            let params: GatewayUpdateParams = parse_params(params_value)?;
            to_json(manager.update(&params.name, params.patch).await?)
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
        assert!(names.contains(&"gateway.supported_services"));
        assert!(names.contains(&"gateway.virtual_server.enable"));
        assert!(names.contains(&"gateway.virtual_server.disable"));
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
            }])
            .await;

        let value = dispatch_with_manager(&manager, "gateway.list", json!({}))
            .await
            .expect("list");

        assert!(value.is_array());
        assert_eq!(value.as_array().expect("array").len(), 1);
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
        assert!(list
            .as_array()
            .expect("array")
            .iter()
            .any(|server| server["id"] == "plex" && server["enabled"] == false));
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
                "url": "http://127.0.0.1:9001",
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
