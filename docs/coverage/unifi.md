# UniFi API Coverage

**Last updated:** 2026-04-09  
**Source spec:** `docs/upstream-api/unifi.md`  
**SDK surface:** `crates/lab-apis/src/unifi/client.rs` (20 public methods: 13 typed wrappers + 7 generic helpers)  
**Shared dispatch layer:** `crates/lab/src/dispatch/unifi.rs` + `crates/lab/src/dispatch/unifi/` (catalog, client, params, dispatch, domain modules)  
**MCP actions:** `crates/lab/src/mcp/services/unifi.rs` (thin adapter over `dispatch::unifi`)  
**CLI surface:** `crates/lab/src/cli/unifi.rs` (generic `action` + `key=value` params)  
**API handler:** `crates/lab/src/api/services/unifi.rs` (thin adapter over the shared dispatch layer)

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented in code |
| ⚠️ | Implemented, but destructive |
| ⬜ | Not live-tested in this workspace |
| — | Not applicable |

> UniFi is exposed as a single action dispatcher across MCP, CLI, and API, but
> the shared execution path now lives in `crates/lab/src/dispatch/unifi.rs`.
> The implementation remains action-centric, not subcommand-centric. The safe
> smoke-tests run in this workspace were limited to discovery and dispatch
> routing because no UniFi controller credentials are configured here.
> Destructive actions were intentionally not exercised.

## Implementation Model

- The SDK keeps the typed read-only wrappers for the common inventory calls.
- The rest of the surface is handled by generic JSON helpers in `UnifiClient`
  so the shared `dispatch::unifi` dispatcher can serve MCP, CLI, and API.
- Action names are the contract: `system.info`, `devices.list`, `networks.update`,
  and so on.
- `help` is built in to the dispatcher and is not a service endpoint.

## SDK Surface

| Method | Purpose |
|--------|---------|
| `info()` | Application metadata |
| `sites_list()` | List sites |
| `devices_list(site_id)` | List adopted devices |
| `device_get(site_id, device_id)` | Inspect one adopted device |
| `device_stats_latest(site_id, device_id)` | Latest device statistics |
| `pending_devices_list()` | List devices pending adoption |
| `clients_list(site_id)` | List connected clients |
| `client_get(site_id, client_id)` | Inspect one client |
| `networks_list(site_id)` | List networks |
| `network_get(site_id, network_id)` | Inspect one network |
| `network_references(site_id, network_id)` | Reference graph for one network |
| `wifi_broadcasts_list(site_id)` | List WiFi broadcasts |
| `wifi_broadcast_get(site_id, wifi_broadcast_id)` | Inspect one WiFi broadcast |
| `get_value(path)` / `get_value_query(path, query)` | Generic GET helpers |
| `post_value(path, body)` | Generic POST helper |
| `put_value(path, body)` | Generic PUT helper |
| `patch_value(path, body)` | Generic PATCH helper |
| `delete_value(path)` / `delete_value_query(path, query)` | Generic DELETE helpers |

## Action Catalog

### System

| Action | SDK helper | Status | Notes |
|--------|------------|--------|-------|
| `system.info` | `info()` | ✅ | Application version and runtime metadata |

### Sites

| Action | SDK helper | Status | Notes |
|--------|------------|--------|-------|
| `sites.list` | `sites_list()` | ✅ | Site inventory |

### Devices

| Action | SDK helper | Status | Notes |
|--------|------------|--------|-------|
| `devices.list` | `devices_list()` | ✅ | Adopted devices for a site |
| `devices.get` | `device_get()` | ✅ | One adopted device |
| `devices.stats` | `device_stats_latest()` | ✅ | Latest device telemetry |
| `devices.create` | `post_value()` | ⚠️ | Adopt a device |
| `devices.port-action` | `post_value()` | ⚠️ | Port-level action, such as power cycle |
| `devices.action` | `post_value()` | ⚠️ | Device action, such as restart |
| `devices.delete` | `delete_value()` | ⚠️ | Unadopt/remove a device |

### Pending Devices

| Action | SDK helper | Status | Notes |
|--------|------------|--------|-------|
| `pending-devices.list` | `pending_devices_list()` | ✅ | Devices waiting for adoption |

### Clients

| Action | SDK helper | Status | Notes |
|--------|------------|--------|-------|
| `clients.list` | `clients_list()` | ✅ | Connected clients for a site |
| `clients.get` | `client_get()` | ✅ | One connected client |
| `clients.action` | `post_value()` | ⚠️ | Guest auth / client control actions |

### Networks

| Action | SDK helper | Status | Notes |
|--------|------------|--------|-------|
| `networks.list` | `networks_list()` | ✅ | Network inventory |
| `network.get` | `network_get()` | ✅ | One network |
| `network.references` | `network_references()` | ✅ | References for a network |
| `networks.create` | `post_value()` | ⚠️ | Create a network |
| `networks.update` | `put_value()` | ⚠️ | Update a network |
| `networks.delete` | `delete_value()` | ⚠️ | Delete a network |

### WiFi Broadcasts

| Action | SDK helper | Status | Notes |
|--------|------------|--------|-------|
| `wifi.broadcasts.list` | `wifi_broadcasts_list()` | ✅ | Broadcast inventory |
| `wifi.broadcasts.get` | `wifi_broadcast_get()` | ✅ | One broadcast |
| `wifi.broadcasts.create` | `post_value()` | ⚠️ | Create an SSID/broadcast |
| `wifi.broadcasts.update` | `put_value()` | ⚠️ | Update an SSID/broadcast |
| `wifi.broadcasts.delete` | `delete_value()` | ⚠️ | Delete an SSID/broadcast |

### Hotspot Vouchers

| Action | SDK helper | Status | Notes |
|--------|------------|--------|-------|
| `hotspot.vouchers.list` | `get_value_query()` | ✅ | Voucher inventory |
| `hotspot.vouchers.create` | `post_value()` | ⚠️ | Generate vouchers |
| `hotspot.vouchers.delete` | `delete_value_query()` | ⚠️ | Delete vouchers by filter |
| `hotspot.vouchers.get` | `get_value()` | ✅ | One voucher |

### Firewall Zones

| Action | SDK helper | Status | Notes |
|--------|------------|--------|-------|
| `firewall.zones.list` | `get_value()` | ✅ | Zone inventory |
| `firewall.zones.get` | `get_value()` | ✅ | One zone |
| `firewall.zones.create` | `post_value()` | ⚠️ | Create a zone |
| `firewall.zones.update` | `put_value()` | ⚠️ | Update a zone |
| `firewall.zones.delete` | `delete_value()` | ⚠️ | Delete a zone |

### Firewall Policies

| Action | SDK helper | Status | Notes |
|--------|------------|--------|-------|
| `firewall.policies.list` | `get_value()` | ✅ | Policy inventory |
| `firewall.policies.get` | `get_value()` | ✅ | One policy |
| `firewall.policies.create` | `post_value()` | ⚠️ | Create a policy |
| `firewall.policies.update` | `put_value()` | ⚠️ | Update a policy |
| `firewall.policies.patch` | `patch_value()` | ⚠️ | Partial update |
| `firewall.policies.ordering.get` | `get_value()` | ✅ | Policy order |
| `firewall.policies.ordering.set` | `put_value()` | ⚠️ | Reorder policies |

### ACL Rules

| Action | SDK helper | Status | Notes |
|--------|------------|--------|-------|
| `acl.rules.list` | `get_value()` | ✅ | ACL inventory |
| `acl.rules.get` | `get_value()` | ✅ | One ACL rule |
| `acl.rules.create` | `post_value()` | ⚠️ | Create an ACL rule |
| `acl.rules.update` | `put_value()` | ⚠️ | Update an ACL rule |
| `acl.rules.delete` | `delete_value()` | ⚠️ | Delete an ACL rule |
| `acl.rules.ordering.get` | `get_value()` | ✅ | ACL order |
| `acl.rules.ordering.set` | `put_value()` | ⚠️ | Reorder ACL rules |

### Switching

| Action | SDK helper | Status | Notes |
|--------|------------|--------|-------|
| `switching.switch-stacks.list` | `get_value()` | ✅ | Switch stack inventory |
| `switching.switch-stacks.get` | `get_value()` | ✅ | One switch stack |
| `switching.mc-lag-domains.list` | `get_value()` | ✅ | MC-LAG domain inventory |
| `switching.mc-lag-domains.get` | `get_value()` | ✅ | One MC-LAG domain |
| `switching.lags.list` | `get_value()` | ✅ | Link aggregation inventory |
| `switching.lags.get` | `get_value()` | ✅ | One LAG |

### DNS Policies

| Action | SDK helper | Status | Notes |
|--------|------------|--------|-------|
| `dns.policies.list` | `get_value()` | ✅ | DNS policy inventory |
| `dns.policies.get` | `get_value()` | ✅ | One DNS policy |
| `dns.policies.create` | `post_value()` | ⚠️ | Create a DNS policy |
| `dns.policies.update` | `put_value()` | ⚠️ | Update a DNS policy |
| `dns.policies.delete` | `delete_value()` | ⚠️ | Delete a DNS policy |

### Traffic Matching Lists

| Action | SDK helper | Status | Notes |
|--------|------------|--------|-------|
| `traffic-matching-lists.list` | `get_value()` | ✅ | Traffic matching list inventory |
| `traffic-matching-lists.get` | `get_value()` | ✅ | One traffic matching list |
| `traffic-matching-lists.create` | `post_value()` | ⚠️ | Create a list |
| `traffic-matching-lists.update` | `put_value()` | ⚠️ | Update a list |
| `traffic-matching-lists.delete` | `delete_value()` | ⚠️ | Delete a list |

### WANs

| Action | SDK helper | Status | Notes |
|--------|------------|--------|-------|
| `wans.list` | `get_value()` | ✅ | WAN inventory |

### VPN

| Action | SDK helper | Status | Notes |
|--------|------------|--------|-------|
| `vpn.site-to-site-tunnels.list` | `get_value()` | ✅ | Site-to-site tunnel inventory |
| `vpn.servers.list` | `get_value()` | ✅ | VPN server inventory |

### Radius Profiles

| Action | SDK helper | Status | Notes |
|--------|------------|--------|-------|
| `radius.profiles.list` | `get_value()` | ✅ | Radius profile inventory |

### Device Tags

| Action | SDK helper | Status | Notes |
|--------|------------|--------|-------|
| `device-tags.list` | `get_value()` | ✅ | Device tag inventory |

### DPI

| Action | SDK helper | Status | Notes |
|--------|------------|--------|-------|
| `dpi.categories.list` | `get_value()` | ✅ | DPI category inventory |
| `dpi.applications.list` | `get_value()` | ✅ | DPI application inventory |

### Countries

| Action | SDK helper | Status | Notes |
|--------|------------|--------|-------|
| `countries.list` | `get_value()` | ✅ | Country inventory |

## Notes

- `UNIFI_URL` should point at the controller root, for example `https://10.1.0.1`.
- `UNIFI_API_KEY` is sent as `X-API-KEY`.
- The client appends `/proxy/network/integration/v1` internally, so callers do
  not need to include that prefix in configuration.
- Safe discovery and wiring checks were confirmed through the CLI and MCP `help`
  path. Live controller calls are still pending because this workspace does not
  have a configured UniFi controller.
