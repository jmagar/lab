use lab::config::DeviceRole;
use lab::device::identity::resolve_runtime_role;

#[test]
fn resolves_master_role_when_master_matches_local_hostname() {
    let resolved = resolve_runtime_role("tootie", Some("tootie")).unwrap();
    assert!(matches!(resolved.role, DeviceRole::Master));
}

#[test]
fn resolves_non_master_role_when_master_differs_from_local_hostname() {
    let resolved = resolve_runtime_role("dookie", Some("tootie")).unwrap();
    assert!(matches!(resolved.role, DeviceRole::NonMaster));
    assert_eq!(resolved.master_host, "tootie");
}

#[test]
fn defaults_first_device_to_master_when_master_is_missing() {
    let resolved = resolve_runtime_role("tootie", None).unwrap();
    assert!(matches!(resolved.role, DeviceRole::Master));
    assert_eq!(resolved.master_host, "tootie");
}
