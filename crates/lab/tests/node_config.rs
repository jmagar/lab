#[test]
fn parses_node_controller_config_block() {
    let raw = r#"
        [node]
        controller = "tootie"
    "#;

    let parsed: lab::config::LabConfig = toml::from_str(raw).unwrap();
    assert_eq!(
        parsed.node.as_ref().unwrap().controller.as_deref(),
        Some("tootie")
    );
}

#[test]
fn defaults_node_config_when_block_missing() {
    let parsed: lab::config::LabConfig = toml::from_str("").unwrap();
    assert!(parsed.node.is_none());
}
