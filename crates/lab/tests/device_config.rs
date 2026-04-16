#[test]
fn parses_device_master_config_block() {
    let raw = r#"
        [device]
        master = "tootie"
    "#;

    let parsed: lab::config::LabConfig = toml::from_str(raw).unwrap();
    assert_eq!(
        parsed.device.as_ref().unwrap().master.as_deref(),
        Some("tootie")
    );
}

#[test]
fn defaults_device_config_when_block_missing() {
    let parsed: lab::config::LabConfig = toml::from_str("").unwrap();
    assert!(parsed.device.is_none() || parsed.device.as_ref().unwrap().master.is_none());
}
