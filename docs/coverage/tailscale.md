# Tailscale API Coverage

**Last updated:** 2026-04-08
**OpenAPI spec:** docs/api-specs/tailscale.openapi.yaml
**OpenAPI version:** 3.1.0
**API version:** v2
**Paths:** 85
**Servers:** https://api.tailscale.com/api/v2
**Security schemes:** bearerAuth

## Legend

| Symbol | Meaning |
|--------|---------|
| ⬜ | Not implemented yet; rows are spec inventory only |
| - | Not applicable / not represented in the spec |

The source spec is the contract. This document is the implementation planning aid.

## Devices

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| DELETE | /device/{deviceId} | deleteDevice | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /device/{deviceId} | getDevice | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /device/{deviceId}/attributes | getDevicePostureAttributes | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /device/{deviceId}/attributes/{attributeKey} | deleteCustomDevicePostureAttributes | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /device/{deviceId}/attributes/{attributeKey} | setCustomDevicePostureAttributes | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /device/{deviceId}/authorized | authorizeDevice | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /device/{deviceId}/expire | expireDeviceKey | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /device/{deviceId}/ip | setDeviceIp | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /device/{deviceId}/key | updateDeviceKey | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /device/{deviceId}/name | setDeviceName | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /device/{deviceId}/routes | listDeviceRoutes | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /device/{deviceId}/routes | setDeviceRoutes | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /device/{deviceId}/tags | setDeviceTags | ⬜ | ⬜ | ⬜ | ⬜ |
| PATCH | /tailnet/{tailnet}/device-attributes | batchUpdateCustomDevicePostureAttributes | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tailnet/{tailnet}/devices | listTailnetDevices | ⬜ | ⬜ | ⬜ | ⬜ |

## PolicyFile

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /tailnet/{tailnet}/acl | getPolicyFile | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /tailnet/{tailnet}/acl | setPolicyFile | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /tailnet/{tailnet}/acl/preview | previewRuleMatches | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /tailnet/{tailnet}/acl/validate | validateAndTestPolicyFile | ⬜ | ⬜ | ⬜ | ⬜ |

## Keys

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /tailnet/{tailnet}/keys | listTailnetKeys | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /tailnet/{tailnet}/keys | createKey | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /tailnet/{tailnet}/keys/{keyId} | deleteKey | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tailnet/{tailnet}/keys/{keyId} | getKey | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /tailnet/{tailnet}/keys/{keyId} | setKey | ⬜ | ⬜ | ⬜ | ⬜ |

## DNS

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /tailnet/{tailnet}/dns/configuration | getDnsConfiguration | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /tailnet/{tailnet}/dns/configuration | setDnsConfiguration | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tailnet/{tailnet}/dns/nameservers | listDnsNameservers | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /tailnet/{tailnet}/dns/nameservers | setDnsNameservers | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tailnet/{tailnet}/dns/preferences | getDnsPreferences | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /tailnet/{tailnet}/dns/preferences | setDnsPreferences | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tailnet/{tailnet}/dns/searchpaths | listDnsSearchPaths | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /tailnet/{tailnet}/dns/searchpaths | setDnsSearchPaths | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /tailnet/{tailnet}/dns/split-dns | getSplitDns | ⬜ | ⬜ | ⬜ | ⬜ |
| PATCH | /tailnet/{tailnet}/dns/split-dns | updateSplitDns | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /tailnet/{tailnet}/dns/split-dns | setSplitDns | ⬜ | ⬜ | ⬜ | ⬜ |

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
| GET | /tailnet/{tailnet}/users | listUsers | ⬜ | ⬜ | ⬜ | ⬜ |
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
| GET | /tailnet/{tailnet}/settings | getTailnetSettings | ⬜ | ⬜ | ⬜ | ⬜ |
| PATCH | /tailnet/{tailnet}/settings | updateTailnetSettings | ⬜ | ⬜ | ⬜ | ⬜ |

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
