# Tailscale API Coverage

**Last updated:** 2026-04-14
**OpenAPI spec:** docs/api-specs/tailscale.openapi.yaml
**OpenAPI version:** 3.1.0
**API version:** v2
**Paths:** 85
**Servers:** https://api.tailscale.com/api/v2
**Security schemes:** bearerAuth

## Implementation Status

The Tailscale service is **fully onboarded** with a partial SDK covering the most
commonly used endpoints. The shared dispatch layer (`crates/lab/src/dispatch/tailscale/`)
is fully implemented with catalog, client, params, and dispatch modules.

### Surface wiring

| Surface | Status | Notes |
|---------|--------|-------|
| SDK (`lab-apis`) | ✅ (partial) | `TailscaleClient` in `crates/lab-apis/src/tailscale/client.rs` |
| Dispatch layer | ✅ | `crates/lab/src/dispatch/tailscale/` — catalog, client, params, dispatch + entrypoint |
| MCP | ✅ | `crates/lab/src/mcp/services/tailscale.rs` — thin delegate to dispatch layer |
| CLI | ✅ | `crates/lab/src/cli/tailscale.rs` — thin shim, action+params dispatch |
| API | ✅ | `crates/lab/src/api/services/tailscale.rs` — axum route calling `dispatch_with_client` |

All surfaces delegate to the shared dispatch layer at
`crates/lab/src/dispatch/tailscale/dispatch.rs`.

### Implemented Actions (24 total)

| Action | SDK Method | Endpoint | MCP | CLI | API | Destructive |
|--------|-----------|---------|-----|-----|-----|-------------|
| `help` | N/A | N/A | ✅ | ✅ | ✅ | No |
| `schema` | N/A | N/A | ✅ | ✅ | ✅ | No |
| `device.list` | `devices_list()` | GET /tailnet/{tailnet}/devices | ✅ | ✅ | ✅ | No |
| `device.get` | `device_get()` | GET /device/{deviceId} | ✅ | ✅ | ✅ | No |
| `device.delete` | `device_delete()` | DELETE /device/{deviceId} | ✅ | ✅ | ✅ | **Yes** |
| `device.authorize` | `device_authorize()` | POST /device/{deviceId}/authorized | ✅ | ✅ | ✅ | No |
| `device.routes-get` | `device_routes_get()` | GET /device/{deviceId}/routes | ✅ | ✅ | ✅ | No |
| `device.routes-set` | `device_routes_set()` | POST /device/{deviceId}/routes | ✅ | ✅ | ✅ | No |
| `device.tag` | `device_tag()` | POST /device/{deviceId}/tags | ✅ | ✅ | ✅ | No |
| `device.expire` | `device_expire()` | POST /device/{deviceId}/expire | ✅ | ✅ | ✅ | **Yes** |
| `key.list` | `keys_list()` | GET /tailnet/{tailnet}/keys | ✅ | ✅ | ✅ | No |
| `key.get` | `key_get()` | GET /tailnet/{tailnet}/keys/{keyId} | ✅ | ✅ | ✅ | No |
| `key.delete` | `key_delete()` | DELETE /tailnet/{tailnet}/keys/{keyId} | ✅ | ✅ | ✅ | **Yes** |
| `key.create` | `key_create()` | POST /tailnet/{tailnet}/keys | ✅ | ✅ | ✅ | No |
| `dns.nameservers` | `dns_nameservers()` | GET /tailnet/{tailnet}/dns/nameservers | ✅ | ✅ | ✅ | No |
| `dns.search_paths` | `dns_search_paths()` | GET /tailnet/{tailnet}/dns/searchpaths | ✅ | ✅ | ✅ | No |
| `dns.split-get` | `dns_split_get()` | GET /tailnet/{tailnet}/dns/split-dns | ✅ | ✅ | ✅ | No |
| `dns.split-set` | `dns_split_set()` | PUT /tailnet/{tailnet}/dns/split-dns | ✅ | ✅ | ✅ | No |
| `acl.get` | `acl_get()` | GET /tailnet/{tailnet}/acl | ✅ | ✅ | ✅ | No |
| `acl.validate` | `acl_validate()` | POST /tailnet/{tailnet}/acl/validate | ✅ | ✅ | ✅ | No |
| `acl.set` | `acl_set()` (with validate-first guard) | POST /tailnet/{tailnet}/acl | ✅ | ✅ | ✅ | No |
| `user.list` | `user_list()` | GET /tailnet/{tailnet}/users | ✅ | ✅ | ✅ | No |
| `tailnet.settings-get` | `tailnet_settings_get()` | GET /tailnet/{tailnet}/settings | ✅ | ✅ | ✅ | No |
| `tailnet.settings-patch` | `tailnet_settings_patch()` | PATCH /tailnet/{tailnet}/settings | ✅ | ✅ | ✅ | No |

Built-in actions `help` and `schema` are handled in `dispatch.rs` before the
service-specific action match and are available on all surfaces.

### Safety: acl.set validate-first guard

The `acl.set` dispatch arm in `dispatch.rs` always calls `acl_validate()` with the same
policy body first. If validation returns a non-null `message` field or a non-empty `errors`
array, dispatch returns a `validation_failed` error **without** calling `acl_set()`.
This prevents blind policy overwrites that could lock all devices out of the tailnet.

### Action Parameters

**`device.get`** — required: `device_id` (string, nodeId or legacy numeric ID).

**`device.delete`** — required: `device_id` (string). Destructive: requires `confirm: true`
in JSON params or `-y`/`--yes` on CLI.

**`device.authorize`** — required: `device_id` (string), `authorized` (bool).

**`device.routes-get`** — required: `device_id` (string).

**`device.routes-set`** — required: `device_id` (string), `routes` (array of CIDR strings).

**`device.tag`** — required: `device_id` (string), `tags` (array of tag strings, e.g. `["tag:server"]`).

**`device.expire`** — required: `device_id` (string). Destructive: requires `confirm: true`
or `-y`. Forces device re-authentication.

**`key.get`** — required: `key_id` (string).

**`key.delete`** — required: `key_id` (string). Destructive: requires `confirm: true` or `-y`.

**`key.create`** — required: `capabilities` (object). Optional: `expiry_seconds` (integer),
`description` (string). Response includes the key string.

**`acl.validate`** — required: `policy` (object, HuJSON policy).

**`acl.set`** — required: `policy` (object, HuJSON policy). Validates first; returns
`validation_failed` error if validation fails, preventing blind overwrites.

**`dns.split-set`** — required: `config` (object, domain→resolver list mapping).

**`tailnet.settings-patch`** — required: `settings` (partial settings object).

**`schema`** — required: `action` (string, action name to describe).

### Return Types

| Action | Return Type |
|--------|-------------|
| `device.list` | `DeviceList` (`{ devices: Device[] }`) |
| `device.get` | `Device` |
| `device.delete` | `null` |
| `device.authorize` | `null` |
| `device.routes-get` | Object (`{ advertisedRoutes: string[], enabledRoutes: string[] }`) |
| `device.routes-set` | Object (same shape as routes-get) |
| `device.tag` | `null` |
| `device.expire` | `null` |
| `key.list` | `KeyList` (`{ keys: AuthKey[] }`) |
| `key.get` | `AuthKey` |
| `key.delete` | `null` |
| `key.create` | `AuthKey` (includes `key` string) |
| `dns.nameservers` | `DnsNameservers` (`{ dns: string[] }`) |
| `dns.search_paths` | `DnsSearchPaths` (`{ searchPaths: string[] }`) |
| `dns.split-get` | Object (domain→resolver list mapping) |
| `dns.split-set` | Object (same shape as split-get) |
| `acl.get` | Object (HuJSON policy) |
| `acl.validate` | Object (`{}` on success, or `{ errors: [...], message?: string }` on failure) |
| `acl.set` | Object (applied HuJSON policy) |
| `user.list` | Object (`{ users: User[] }`) |
| `tailnet.settings-get` | Object (settings object) |
| `tailnet.settings-patch` | Object (updated settings object) |

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented |
| ⬜ | Not implemented yet; rows are spec inventory only |
| - | Not applicable / not represented in the spec |

The source spec is the contract. This document is the implementation planning aid.

## Devices

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| DELETE | /device/{deviceId} | `device_delete` | ✅ | ✅ | ✅ | ✅ |
| GET | /device/{deviceId} | `device_get` | ✅ | ✅ | ✅ | ✅ |
| GET | /device/{deviceId}/attributes | getDevicePostureAttributes | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /device/{deviceId}/attributes/{attributeKey} | deleteCustomDevicePostureAttributes | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /device/{deviceId}/attributes/{attributeKey} | setCustomDevicePostureAttributes | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /device/{deviceId}/authorized | `device_authorize` | ✅ | ✅ | ✅ | ✅ |
| POST | /device/{deviceId}/expire | `device_expire` | ✅ | ✅ | ✅ | ✅ |
| POST | /device/{deviceId}/ip | setDeviceIp | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /device/{deviceId}/key | updateDeviceKey | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /device/{deviceId}/name | setDeviceName | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /device/{deviceId}/routes | `device_routes_get` | ✅ | ✅ | ✅ | ✅ |
| POST | /device/{deviceId}/routes | `device_routes_set` | ✅ | ✅ | ✅ | ✅ |
| POST | /device/{deviceId}/tags | `device_tag` | ✅ | ✅ | ✅ | ✅ |
| PATCH | /tailnet/{tailnet}/device-attributes | batchUpdateCustomDevicePostureAttributes | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tailnet/{tailnet}/devices | `devices_list` | ✅ | ✅ | ✅ | ✅ |

## PolicyFile

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /tailnet/{tailnet}/acl | `acl_get` | ✅ | ✅ | ✅ | ✅ |
| POST | /tailnet/{tailnet}/acl | `acl_set` | ✅ | ✅ | ✅ | ✅ |
| POST | /tailnet/{tailnet}/acl/preview | previewRuleMatches | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /tailnet/{tailnet}/acl/validate | `acl_validate` | ✅ | ✅ | ✅ | ✅ |

## Keys

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /tailnet/{tailnet}/keys | `keys_list` | ✅ | ✅ | ✅ | ✅ |
| POST | /tailnet/{tailnet}/keys | `key_create` | ✅ | ✅ | ✅ | ✅ |
| DELETE | /tailnet/{tailnet}/keys/{keyId} | `key_delete` | ✅ | ✅ | ✅ | ✅ |
| GET | /tailnet/{tailnet}/keys/{keyId} | `key_get` | ✅ | ✅ | ✅ | ✅ |
| PUT | /tailnet/{tailnet}/keys/{keyId} | setKey | ⬜ | ⬜ | ⬜ | ⬜ |

## DNS

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /tailnet/{tailnet}/dns/configuration | getDnsConfiguration | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /tailnet/{tailnet}/dns/configuration | setDnsConfiguration | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tailnet/{tailnet}/dns/nameservers | `dns_nameservers` | ✅ | ✅ | ✅ | ✅ |
| POST | /tailnet/{tailnet}/dns/nameservers | setDnsNameservers | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tailnet/{tailnet}/dns/preferences | getDnsPreferences | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /tailnet/{tailnet}/dns/preferences | setDnsPreferences | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tailnet/{tailnet}/dns/searchpaths | `dns_search_paths` | ✅ | ✅ | ✅ | ✅ |
| POST | /tailnet/{tailnet}/dns/searchpaths | setDnsSearchPaths | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tailnet/{tailnet}/dns/split-dns | `dns_split_get` | ✅ | ✅ | ✅ | ✅ |
| PATCH | /tailnet/{tailnet}/dns/split-dns | updateSplitDns | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /tailnet/{tailnet}/dns/split-dns | `dns_split_set` | ✅ | ✅ | ✅ | ✅ |

## Logging

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| POST | /tailnet/{tailnet}/aws-external-id | getAwsExternalId | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /tailnet/{tailnet}/aws-external-id/{id}/validate-aws-trust-policy | validateAwsExternalId | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tailnet/{tailnet}/logging/configuration | listConfigurationAuditLogs | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tailnet/{tailnet}/logging/network | listNetworkFlowLogs | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /tailnet/{tailnet}/logging/{logType}/stream | disableLogStreaming | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tailnet/{tailnet}/logging/{logType}/stream | getLogStreamingConfiguration | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /tailnet/{tailnet}/logging/{logType}/stream | setLogStreamingConfiguration | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tailnet/{tailnet}/logging/{logType}/stream/status | getLogStreamingStatus | ⬜ | ⬜ | ⬜ | ⬜ |

## Users

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /tailnet/{tailnet}/users | `user_list` | ✅ | ✅ | ✅ | ✅ |
| GET | /users/{userId} | getUser | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /users/{userId}/approve | approveUser | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /users/{userId}/delete | deleteUser | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /users/{userId}/restore | restoreUser | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /users/{userId}/role | updateUserRole | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /users/{userId}/suspend | suspendUser | ⬜ | ⬜ | ⬜ | ⬜ |

## UserInvites

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /tailnet/{tailnet}/user-invites | listUserInvites | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /tailnet/{tailnet}/user-invites | createUserInvites | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /user-invites/{userInviteId} | deleteUserInvite | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /user-invites/{userInviteId} | getUserInvite | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /user-invites/{userInviteId}/resend | resendUserInvite | ⬜ | ⬜ | ⬜ | ⬜ |

## DeviceInvites

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| POST | /device-invites/-/accept | acceptDeviceInvite | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /device-invites/{deviceInviteId} | deleteDeviceInvite | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /device-invites/{deviceInviteId} | getDeviceInvite | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /device-invites/{deviceInviteId}/resend | resendDeviceInvite | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /device/{deviceId}/device-invites | listDeviceInvites | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /device/{deviceId}/device-invites | createDeviceInvites | ⬜ | ⬜ | ⬜ | ⬜ |

## DevicePosture

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| DELETE | /posture/integrations/{id} | deletePostureIntegration | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /posture/integrations/{id} | getPostureIntegration | ⬜ | ⬜ | ⬜ | ⬜ |
| PATCH | /posture/integrations/{id} | updatePostureIntegration | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tailnet/{tailnet}/posture/integrations | getPostureIntegrations | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /tailnet/{tailnet}/posture/integrations | createPostureIntegration | ⬜ | ⬜ | ⬜ | ⬜ |

## Contacts

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /tailnet/{tailnet}/contacts | getContacts | ⬜ | ⬜ | ⬜ | ⬜ |
| PATCH | /tailnet/{tailnet}/contacts/{contactType} | updateContact | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /tailnet/{tailnet}/contacts/{contactType}/resend-verification-email | resendContactVerificationEmail | ⬜ | ⬜ | ⬜ | ⬜ |

## Webhooks

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /tailnet/{tailnet}/webhooks | listWebhooks | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /tailnet/{tailnet}/webhooks | createWebhook | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /webhooks/{endpointId} | deleteWebhook | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /webhooks/{endpointId} | getWebhook | ⬜ | ⬜ | ⬜ | ⬜ |
| PATCH | /webhooks/{endpointId} | updateWebhook | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /webhooks/{endpointId}/rotate | rotateWebhookSecret | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /webhooks/{endpointId}/test | testWebhook | ⬜ | ⬜ | ⬜ | ⬜ |

## TailnetSettings

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /tailnet/{tailnet}/settings | `tailnet_settings_get` | ✅ | ✅ | ✅ | ✅ |
| PATCH | /tailnet/{tailnet}/settings | `tailnet_settings_patch` | ✅ | ✅ | ✅ | ✅ |

## Services

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /tailnet/{tailnet}/services | listServices | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /tailnet/{tailnet}/services/{serviceName} | deleteService | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tailnet/{tailnet}/services/{serviceName} | getService | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /tailnet/{tailnet}/services/{serviceName} | updateService | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tailnet/{tailnet}/services/{serviceName}/device/{deviceId}/approved | getServiceDeviceApproval | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /tailnet/{tailnet}/services/{serviceName}/device/{deviceId}/approved | updateServiceDeviceApproval | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tailnet/{tailnet}/services/{serviceName}/devices | listServiceHosts | ⬜ | ⬜ | ⬜ | ⬜ |
