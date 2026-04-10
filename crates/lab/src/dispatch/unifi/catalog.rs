use lab_apis::core::action::ActionSpec;

use super::{
    acl, clients, devices, dns, firewall, hotspot, misc, networks, switching, traffic, wifi,
};

/// Combined action catalog for the `UniFi` service.
pub fn actions() -> &'static [ActionSpec] {
    static ACTIONS: std::sync::LazyLock<&'static [ActionSpec]> = std::sync::LazyLock::new(|| {
        let mut all = vec![ActionSpec {
            name: "help",
            description: "Show this action catalog",
            destructive: false,
            returns: "Catalog",
            params: &[],
        }];
        all.extend_from_slice(misc::ACTIONS);
        all.extend_from_slice(devices::ACTIONS);
        all.extend_from_slice(clients::ACTIONS);
        all.extend_from_slice(networks::ACTIONS);
        all.extend_from_slice(wifi::ACTIONS);
        all.extend_from_slice(hotspot::ACTIONS);
        all.extend_from_slice(firewall::ACTIONS);
        all.extend_from_slice(acl::ACTIONS);
        all.extend_from_slice(switching::ACTIONS);
        all.extend_from_slice(dns::ACTIONS);
        all.extend_from_slice(traffic::ACTIONS);
        Vec::leak(all)
    });
    &ACTIONS
}
