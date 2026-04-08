# UniFi API Coverage

**Last updated:** 2026-04-08  
**Source spec:** `docs/upstream-api/unifi.md`  
**SDK surface:** `crates/lab-apis/src/unifi/` (1 module, 13 methods)  
**MCP actions:** `crates/lab/src/mcp/services/unifi.rs` (13 actions + built-in `help`)  
**CLI subcommands:** `crates/lab/src/cli/unifi.rs` (13 subcommands)  
**HTTP API handler:** `crates/lab/src/api/services/unifi.rs` (single dispatch → same action set as MCP)

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented in code |
| ⬜ | Not yet implemented, or wired but not live-tested |
| — | Not applicable |

> The UniFi work done so far is intentionally read-only. It covers inventory and health
> checks only: application info, sites, adopted devices, pending devices, and connected
> clients. Mutating actions, network config, WiFi, ACL, firewall, and other management
> endpoints remain unimplemented.

## Implemented Slice

| Group | Method | Endpoint | SDK | MCP | CLI | API |
|-------|--------|----------|-----|-----|-----|-----|
| System | GET | `/proxy/network/integration/v1/info` | `info()` | ✅ | ⬜ | ⬜ | ⬜ |
| Sites | GET | `/proxy/network/integration/v1/sites` | `sites_list()` | ✅ | ⬜ | ⬜ | ⬜ |
| Devices | GET | `/proxy/network/integration/v1/sites/{siteId}/devices` | `devices_list(site_id)` | ✅ | ⬜ | ⬜ | ⬜ |
| Devices | GET | `/proxy/network/integration/v1/sites/{siteId}/devices/{deviceId}` | `device_get(site_id, device_id)` | ✅ | ⬜ | ⬜ | ⬜ |
| Devices | GET | `/proxy/network/integration/v1/sites/{siteId}/devices/{deviceId}/statistics/latest` | `device_stats_latest(site_id, device_id)` | ✅ | ⬜ | ⬜ | ⬜ |
| Pending Devices | GET | `/proxy/network/integration/v1/pending-devices` | `pending_devices_list()` | ✅ | ⬜ | ⬜ | ⬜ |
| Clients | GET | `/proxy/network/integration/v1/sites/{siteId}/clients` | `clients_list(site_id)` | ✅ | ⬜ | ⬜ | ⬜ |
| Clients | GET | `/proxy/network/integration/v1/sites/{siteId}/clients/{clientId}` | `client_get(site_id, client_id)` | ✅ | ⬜ | ⬜ | ⬜ |
| Networks | GET | `/proxy/network/integration/v1/sites/{siteId}/networks` | `networks_list(site_id)` | ✅ | ⬜ | ⬜ | ⬜ |
| Networks | GET | `/proxy/network/integration/v1/sites/{siteId}/networks/{networkId}` | `network_get(site_id, network_id)` | ✅ | ⬜ | ⬜ | ⬜ |
| Networks | GET | `/proxy/network/integration/v1/sites/{siteId}/networks/{networkId}/references` | `network_references(site_id, network_id)` | ✅ | ⬜ | ⬜ | ⬜ |
| WiFi Broadcasts | GET | `/proxy/network/integration/v1/sites/{siteId}/wifi/broadcasts` | `wifi_broadcasts_list(site_id)` | ✅ | ⬜ | ⬜ | ⬜ |
| WiFi Broadcasts | GET | `/proxy/network/integration/v1/sites/{siteId}/wifi/broadcasts/{wifiBroadcastId}` | `wifi_broadcast_get(site_id, wifi_broadcast_id)` | ✅ | ⬜ | ⬜ | ⬜ |

## Unimplemented Remainder

| Group | Coverage |
|-------|----------|
| Device actions | Adopt/unadopt, port actions, device actions |
| Networks | Create, update, delete, inspect, references |
| WiFi broadcasts | Read/write SSID configuration |
| Client actions | Guest authorization actions |
| ACL / firewall / advanced config | Not started |

## Notes

- `UNIFI_URL` should point at the controller root, for example `https://10.1.0.1`.
- `UNIFI_API_KEY` is sent as `X-API-KEY`.
- The client appends `/proxy/network/integration/v1` internally, so callers do not
  need to include that prefix in configuration.
