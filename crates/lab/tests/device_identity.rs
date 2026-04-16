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

#[test]
fn treats_short_hostname_and_fqdn_as_same_device() {
    let resolved = resolve_runtime_role("tootie", Some("tootie.tailnet.ts.net")).unwrap();
    assert!(matches!(resolved.role, DeviceRole::Master));
}

#[test]
fn does_not_treat_ip_addresses_with_same_first_octet_as_same_device() {
    let resolved = resolve_runtime_role("100.64.0.1", Some("100.88.0.2")).unwrap();
    assert!(matches!(resolved.role, DeviceRole::NonMaster));
}
