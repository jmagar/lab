# UniFi API Coverage

**Last updated:** 2026-04-13
**Source spec:** `docs/upstream-api/unifi.md`
**SDK surface:** `crates/lab-apis/src/unifi/client.rs` (20 public methods: 13 typed wrappers + 7 generic helpers)
**Shared dispatch layer:** `crates/lab/src/dispatch/unifi.rs` + `crates/lab/src/dispatch/unifi/` (catalog, client, params, dispatch, and 10 domain modules)
**MCP actions:** delegated directly from `dispatch::unifi::dispatch` (no separate MCP service module)
**CLI surface:** `crates/lab/src/cli/unifi.rs` (action string + `key=value` trailing params)
**API handler:** `crates/lab/src/api/services/unifi.rs` (thin adapter over the shared dispatch layer)
**Total actions:** 81 (1 built-in `help` + 80 resource actions)

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented in code |
| ⚠️ | Implemented, but destructive |
| — | Not applicable |

> UniFi is exposed as a single action dispatcher across MCP, CLI, and API. All shared
> execution logic lives in `crates/lab/src/dispatch/unifi/`. The implementation is
> action-centric. No live controller calls were made during development of this workspace
> because no UniFi credentials are configured here. Destructive actions were not exercised.

## Implementation Model

- Typed SDK wrappers cover the most-common read-only inventory calls.
- Remaining endpoints use the generic JSON helpers (`get_value`, `post_value`, etc.) so the
  shared dispatcher can reach any API path without requiring a new typed method per endpoint.
- Action names are the contract: `system.info`, `devices.list`, `networks.update`, and so on.
- `help` and `schema` are built-in to the dispatcher and are not service endpoints.
- 10 domain modules partition the action catalog: `misc`, `devices`, `clients`, `networks`,
  `wifi`, `hotspot`, `firewall`, `acl`, `switching`, `dns`, `traffic`.

## SDK Surface (`crates/lab-apis/src/unifi/client.rs`)

| Method | Purpose |
|--------|---------|
| `info()` | Application metadata |
| `sites_list()` | List sites |
| `devices_list(site_id)` | List adopted devices |
| `device_get(site_id, device_id)` | Inspect one adopted device |
| `device_stats_latest(site_id, device_id)` | Latest device statistics |
| `pending_devices_list()` | List devices pending adoption |
| `clients_list(site_id)` | List active clients |
| `client_get(site_id, client_id)` | Inspect one client |
| `networks_list(site_id)` | List networks |
| `network_get(site_id, network_id)` | Inspect one network |
| `network_references(site_id, network_id)` | Reference graph for one network |
| `wifi_broadcasts_list(site_id)` | List Wi-Fi broadcasts |
| `wifi_broadcast_get(site_id, wifi_broadcast_id)` | Inspect one Wi-Fi broadcast |
| `get_value(path)` | Generic GET helper |
| `get_value_query(path, query)` | Generic GET helper with query params |
| `post_value(path, body)` | Generic POST helper |
| `put_value(path, body)` | Generic PUT helper |
| `patch_value(path, body)` | Generic PATCH helper |
| `delete_value(path)` | Generic DELETE helper |
| `delete_value_query(path, query)` | Generic DELETE helper with query params |

All generic helpers prepend `/proxy/network/integration/v1` internally.

## Action Catalog

### System

| Action | SDK helper | Destructive | Params |
|--------|------------|-------------|--------|
| `system.info` | `info()` | No | none |

### Sites

| Action | SDK helper | Destructive | Params |
|--------|------------|-------------|--------|
| `sites.list` | `sites_list()` | No | none |

### WANs

| Action | SDK helper | Destructive | Params |
|--------|------------|-------------|--------|
| `wans.list` | `get_value()` | No | `site_id` (required) |
| `wan.get` | `get_value()` | No | `site_id`, `wan_id` (both required) |

### Devices

| Action | SDK helper | Destructive | Params |
|--------|------------|-------------|--------|
| `devices.list` | `devices_list()` | No | `site_id` (required) |
| `devices.get` | `device_get()` | No | `site_id`, `device_id` (both required) |
| `devices.stats` | `device_stats_latest()` | No | `site_id`, `device_id` (both required) |
| `pending-devices.list` | `pending_devices_list()` | No | none |
| `devices.create` | `post_value()` | ⚠️ | `site_id`, `mac_address` (required); `ignore_device_limit` (optional bool) |
| `devices.port-action` | `post_value()` | ⚠️ | `site_id`, `device_id`, `port_idx` (integer), `action` (all required) |
| `devices.action` | `post_value()` | ⚠️ | `site_id`, `device_id`, `action` (all required) |
| `devices.delete` | `delete_value()` | ⚠️ | `site_id`, `device_id` (both required) |
| `device.update` | `put_value()` | ⚠️ | `site_id`, `device_id` (required); body fields passed through |

### Clients

| Action | SDK helper | Destructive | Params |
|--------|------------|-------------|--------|
| `clients.list` | `clients_list()` | No | `site_id` (required) |
| `clients.get` | `client_get()` | No | `site_id`, `client_id` (both required) |
| `clients.action` | `post_value()` | ⚠️ | `site_id`, `client_id`, `action` (all required) |
| `client.history` | `get_value()` | No | `site_id`, `client_mac` (both required) |
| `client.block` | `post_value()` | ⚠️ | `site_id`, `client_mac` (both required) |
| `client.unblock` | `post_value()` | No | `site_id`, `client_mac` (both required) |

### Networks

| Action | SDK helper | Destructive | Params |
|--------|------------|-------------|--------|
| `networks.list` | `networks_list()` | No | `site_id` (required); `offset`, `limit`, `filter` (optional query) |
| `networks.get` | `network_get()` | No | `site_id`, `network_id` (both required) |
| `networks.references` | `network_references()` | No | `site_id`, `network_id` (both required) |
| `networks.create` | `post_value()` | ⚠️ | `site_id` (required); additional body fields passed through |
| `networks.update` | `put_value()` | ⚠️ | `site_id`, `network_id` (required); body fields passed through |
| `networks.delete` | `delete_value()` | ⚠️ | `site_id`, `network_id` (both required) |

### Wi-Fi Broadcasts

| Action | SDK helper | Destructive | Params |
|--------|------------|-------------|--------|
| `wifi.broadcasts.list` | `get_value()` | No | `site_id` (required) |
| `wifi.broadcasts.get` | `get_value()` | No | `site_id`, `wifi_broadcast_id` (both required) |
| `wifi.broadcasts.create` | `post_value()` | ⚠️ | `site_id` (required); body fields passed through |
| `wifi.broadcasts.update` | `put_value()` | ⚠️ | `site_id`, `wifi_broadcast_id` (required); body fields passed through |
| `wifi.broadcasts.delete` | `delete_value()` | ⚠️ | `site_id`, `wifi_broadcast_id` (both required) |
| `wifi.update` | `put_value()` | ⚠️ | `site_id`, `wifi_id` (required); body fields passed through |

### Hotspot Vouchers

| Action | SDK helper | Destructive | Params |
|--------|------------|-------------|--------|
| `hotspot.vouchers.list` | `get_value_query()` | No | `site_id` (required); `offset`, `limit`, `filter` (optional query) |
| `hotspot.vouchers.get` | `get_value()` | No | `site_id`, `voucher_id` (both required) |
| `hotspot.vouchers.create` | `post_value()` | ⚠️ | `site_id` (required); body fields passed through |
| `hotspot.vouchers.delete` | `delete_value_query()` | ⚠️ | `site_id` (required); `filter` (optional query) |

### Firewall Zones

| Action | SDK helper | Destructive | Params |
|--------|------------|-------------|--------|
| `firewall.zones.list` | `get_value()` | No | `site_id` (required); `offset`, `limit`, `filter` (optional query) |
| `firewall.zones.get` | `get_value()` | No | `site_id`, `firewall_zone_id` (both required) |
| `firewall.zones.create` | `post_value()` | ⚠️ | `site_id` (required); body fields passed through |
| `firewall.zones.update` | `put_value()` | ⚠️ | `site_id`, `firewall_zone_id` (required); body fields passed through |
| `firewall.zones.delete` | `delete_value()` | ⚠️ | `site_id`, `firewall_zone_id` (both required) |

### Firewall Policies

| Action | SDK helper | Destructive | Params |
|--------|------------|-------------|--------|
| `firewall.policies.list` | `get_value()` | No | `site_id` (required); `offset`, `limit`, `filter` (optional query) |
| `firewall.policies.get` | `get_value()` | No | `site_id`, `firewall_policy_id` (both required) |
| `firewall.policies.create` | `post_value()` | ⚠️ | `site_id` (required); body fields passed through |
| `firewall.policies.update` | `put_value()` | ⚠️ | `site_id`, `firewall_policy_id` (required); body fields passed through |
| `firewall.policies.patch` | `patch_value()` | ⚠️ | `site_id`, `firewall_policy_id` (required); body fields passed through |
| `firewall.policies.ordering.get` | `get_value()` | No | `site_id` (required) |
| `firewall.policies.ordering.set` | `put_value()` | ⚠️ | `site_id` (required); body fields passed through |

### ACL Rules

| Action | SDK helper | Destructive | Params |
|--------|------------|-------------|--------|
| `acl.rules.list` | `get_value()` | No | `site_id` (required); `offset`, `limit`, `filter` (optional query) |
| `acl.rules.get` | `get_value()` | No | `site_id`, `acl_rule_id` (both required) |
| `acl.rules.create` | `post_value()` | ⚠️ | `site_id` (required); body fields passed through |
| `acl.rules.update` | `put_value()` | ⚠️ | `site_id`, `acl_rule_id` (required); body fields passed through |
| `acl.rules.delete` | `delete_value()` | ⚠️ | `site_id`, `acl_rule_id` (both required) |
| `acl.rules.ordering.get` | `get_value()` | No | `site_id` (required) |
| `acl.rules.ordering.set` | `put_value()` | ⚠️ | `site_id` (required); body fields passed through |

### Switching

| Action | SDK helper | Destructive | Params |
|--------|------------|-------------|--------|
| `switching.switch-stacks.list` | `get_value()` | No | `site_id` (required) |
| `switching.switch-stacks.get` | `get_value()` | No | `site_id`, `switch_stack_id` (both required) |
| `switching.mc-lag-domains.list` | `get_value()` | No | `site_id` (required) |
| `switching.mc-lag-domains.get` | `get_value()` | No | `site_id`, `mc_lag_domain_id` (both required) |
| `switching.lags.list` | `get_value()` | No | `site_id` (required) |
| `switching.lags.get` | `get_value()` | No | `site_id`, `lag_id` (both required) |

### Port Profiles

| Action | SDK helper | Destructive | Params |
|--------|------------|-------------|--------|
| `port-profile.list` | `get_value()` | No | `site_id` (required) |
| `port-profile.create` | `post_value()` | ⚠️ | `site_id` (required); body fields passed through |
| `port-profile.update` | `put_value()` | ⚠️ | `site_id`, `port_profile_id` (required); body fields passed through |

### DNS Policies

| Action | SDK helper | Destructive | Params |
|--------|------------|-------------|--------|
| `dns.policies.list` | `get_value()` | No | `site_id` (required); `offset`, `limit`, `filter` (optional query) |
| `dns.policies.get` | `get_value()` | No | `site_id`, `dns_policy_id` (both required) |
| `dns.policies.create` | `post_value()` | ⚠️ | `site_id` (required); body fields passed through |
| `dns.policies.update` | `put_value()` | ⚠️ | `site_id`, `dns_policy_id` (required); body fields passed through |
| `dns.policies.delete` | `delete_value()` | ⚠️ | `site_id`, `dns_policy_id` (both required) |

### Traffic Matching Lists

| Action | SDK helper | Destructive | Params |
|--------|------------|-------------|--------|
| `traffic-matching-lists.list` | `get_value()` | No | `site_id` (required); `offset`, `limit`, `filter` (optional query) |
| `traffic-matching-lists.get` | `get_value()` | No | `site_id`, `traffic_matching_list_id` (both required) |
| `traffic-matching-lists.create` | `post_value()` | ⚠️ | `site_id` (required); body fields passed through |
| `traffic-matching-lists.update` | `put_value()` | ⚠️ | `site_id`, `traffic_matching_list_id` (required); body fields passed through |
| `traffic-matching-lists.delete` | `delete_value()` | ⚠️ | `site_id`, `traffic_matching_list_id` (both required) |

### VPN

| Action | SDK helper | Destructive | Params |
|--------|------------|-------------|--------|
| `vpn.site-to-site-tunnels.list` | `get_value()` | No | `site_id` (required) |
| `vpn.servers.list` | `get_value()` | No | `site_id` (required) |

### RADIUS Profiles

| Action | SDK helper | Destructive | Params |
|--------|------------|-------------|--------|
| `radius.profiles.list` | `get_value()` | No | `site_id` (required) |

### Device Tags

| Action | SDK helper | Destructive | Params |
|--------|------------|-------------|--------|
| `device-tags.list` | `get_value()` | No | `site_id` (required) |

### DPI

| Action | SDK helper | Destructive | Params |
|--------|------------|-------------|--------|
| `dpi.categories.list` | `get_value()` | No | `offset`, `limit` (optional query) |
| `dpi.applications.list` | `get_value()` | No | `offset`, `limit` (optional query) |

### Countries

| Action | SDK helper | Destructive | Params |
|--------|------------|-------------|--------|
| `countries.list` | `get_value()` | No | `offset`, `limit` (optional query) |

## Surface Notes

### CLI (`crates/lab/src/cli/unifi.rs`)

Tier-2 shim. `lab unifi <action> [key=value ...]`. Supports `--instance <label>`,
`-y`/`--yes` (skip destructive confirm), and `--dry-run`. The dry-run check runs
twice in the current code (a minor implementation artifact, not a user-visible bug).

### MCP

No dedicated `mcp/services/unifi.rs` dispatch function — the MCP registry wires directly
to `dispatch::unifi::dispatch`. The file `mcp/services/unifi.rs` exists but contains only
tests; it does not export a `dispatch` function.

### API (`crates/lab/src/api/services/unifi.rs`)

`POST /v1/unifi` — single route. Calls `dispatch::unifi::dispatch` via the shared
`handle_action` helper. Does not use a pre-built client from `AppState`; client resolution
happens inside the dispatch layer (via `client_from_instance`).

## Config

| Env var | Required | Purpose |
|---------|----------|---------|
| `UNIFI_URL` | Yes | Controller root URL (e.g. `https://10.1.0.1`) |
| `UNIFI_API_KEY` | Yes | API key sent as `X-API-KEY` header |

The client appends `/proxy/network/integration/v1` internally. Multi-instance support: set
`UNIFI_<LABEL>_URL` and `UNIFI_<LABEL>_API_KEY` for additional instances; select via
`params.instance` (MCP/API) or `--instance` (CLI).
