use lab::config::{LabConfig, NodeRuntimeRole, RestartModel};

#[test]
fn parses_node_controller_config_block() {
    let raw = r#"
        [node]
        controller = "tootie"
    "#;

    let parsed: LabConfig = toml::from_str(raw).unwrap();
    assert_eq!(
        parsed.node.as_ref().unwrap().controller.as_deref(),
        Some("tootie")
    );
}

#[test]
fn defaults_node_config_when_block_missing() {
    let parsed: LabConfig = toml::from_str("").unwrap();
    assert!(parsed.node.is_none());
}

#[test]
fn parses_node_role_controller() {
    let config: LabConfig = toml::from_str(
        r#"
        [node]
        role = "controller"
        controller = "dookie"
    "#,
    )
    .unwrap();
    assert_eq!(config.node.unwrap().role, Some(NodeRuntimeRole::Controller));
}

#[test]
fn parses_node_role_node() {
    let config: LabConfig = toml::from_str(
        r#"
        [node]
        role = "node"
        controller = "dookie"
    "#,
    )
    .unwrap();
    assert_eq!(config.node.unwrap().role, Some(NodeRuntimeRole::Node));
}

#[test]
fn parses_deploy_restart_model_blocks() {
    let raw = r#"
        [deploy.defaults.restart]
        kind = "system_service"
        service = "lab"

        [deploy.hosts.edge.restart]
        kind = "wrapper_command"
        command = ["sudo", "systemctl", "restart", "lab"]
    "#;

    let parsed: LabConfig = toml::from_str(raw).unwrap();
    assert_eq!(
        parsed
            .deploy
            .as_ref()
            .unwrap()
            .defaults
            .as_ref()
            .unwrap()
            .restart,
        Some(RestartModel::SystemService {
            service: "lab".into()
        })
    );
    assert_eq!(
        parsed
            .deploy
            .as_ref()
            .unwrap()
            .hosts
            .get("edge")
            .unwrap()
            .restart,
        Some(RestartModel::WrapperCommand {
            command: vec![
                "sudo".into(),
                "systemctl".into(),
                "restart".into(),
                "lab".into()
            ]
        })
    );
}
