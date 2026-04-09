# Observability

This document is the canonical observability contract for `lab`.

It defines:

- where instrumentation is mandatory
- which structured fields are required
- how caller context flows across boundaries
- what must never be logged
- what must be verified before a service is considered online

This is not optional guidance. Service integrations and shared infrastructure should conform to it.

## Goal

Every user-visible service action should be traceable end to end across:

- CLI dispatch
- MCP dispatch
- HTTP API dispatch
- shared SDK transport
- service health probes

When a request fails, operators should be able to answer:

- which surface invoked it
- which service and action ran
- which instance was targeted
- which outbound request was attempted
- whether the failure happened in validation, auth, transport, or server response handling

## Ownership

Observability is split across two layers:

- `lab` owns caller context and dispatch logging
- `lab-apis` owns outbound request logging and transport failure detail

That means:

- CLI, MCP, and HTTP must log the user-visible action boundary
- `HttpClient` must log every outbound request
- service modules must not invent custom logging formats

## Mandatory Instrumentation Points

The following boundaries must emit structured logs.

### CLI Dispatch

Every CLI service action must emit one dispatch event.

Required fields:

- `surface = "cli"`
- `service`
- `action`
- `elapsed_ms`

Optional when applicable:

- `instance`
- `operation = "health"`
- `kind` on failure

### MCP Dispatch

Every MCP tool action must emit one dispatch event.

Required fields:

- `surface = "mcp"`
- `service`
- `action`
- `elapsed_ms`

Optional when applicable:

- `instance`
- `operation = "health"`
- `kind` on failure

### HTTP API Dispatch

Every HTTP service action must emit one dispatch event.

Required fields:

- `surface = "http"`
- `service`
- `action`
- `elapsed_ms`
- `request_id`

Optional when applicable:

- `instance`
- `operation = "health"`
- `kind` on failure

### Shared Outbound Requests

`lab-apis::core::HttpClient` must emit:

- one `request.start` event before every outbound call
- one `request.finish` event on success
- one `request.error` event on failure

This applies to all shared request helpers, including:

- `get_json`
- `get_json_query`
- `get_void`
- `post_json`
- `post_void`
- `put_json`
- `patch_json`
- `delete`
- `delete_query`

`HttpClient` logs must inherit the caller span from CLI, MCP, or HTTP dispatch.

### Health Probes

Health probes are not normal business actions and must be distinguishable in logs.

When a health check runs, logs must include:

- `operation = "health"`

Health probes should also preserve the normal dispatch and request fields for their surface.

### Destructive Actions

Destructive actions must log:

- intent before execution
- outcome after execution

Intent logs should make it clear which action is about to mutate state. Outcome logs should indicate success or failure.

## Required Fields

### Dispatch Events

All dispatch events must include:

- `surface`
- `service`
- `action`
- `elapsed_ms`

Failure events must also include:

- `kind`

Additional fields when applicable:

- `instance`
- `request_id`
- `operation`

### Request Events

All `HttpClient` request events must include:

- `method`
- `path`
- `host`

`request.finish` must also include:

- `status`
- `elapsed_ms`

`request.error` must also include:

- `elapsed_ms`
- `kind`
- `message`

If the implementation logs a URL, it must be redacted and must not contain secrets or embedded credentials.

## Correlation Rules

Caller context must flow downward.

Rules:

- CLI spans must wrap SDK calls
- MCP spans must wrap SDK calls
- HTTP spans must wrap SDK calls
- `HttpClient` request events must inherit those spans rather than creating detached logs

The practical result should be:

- outbound request logs can be tied back to the invoking surface
- HTTP-originated requests can be tied back to a `request_id`
- multi-instance requests can be tied back to an `instance`

## Error Classification

The public error taxonomy remains the stable contract.

Relevant kinds include:

- `auth_failed`
- `not_found`
- `rate_limited`
- `validation_failed`
- `network_error`
- `server_error`
- `decode_error`
- `internal_error`

Dispatch layers may also emit:

- `unknown_action`
- `unknown_subaction`
- `missing_param`
- `invalid_param`
- `unknown_instance`

Transport failures must preserve enough message detail to distinguish likely classes such as:

- DNS resolution failure
- TCP connection failure
- TLS certificate validation failure
- timeout

Those details may live in the error message while still mapping to the stable `network_error` kind.

## Redaction Rules

The following data must never be logged:

- API keys
- bearer tokens
- passwords
- cookies
- authorization headers
- secret env values

Additional rules:

- do not log full request headers unless explicitly sanitized
- do not log request bodies by default
- do not log query parameters when they contain secrets
- do not echo secrets in doctor output, prompts, or TUI flows

## Level Rules

Use these level conventions consistently:

- `INFO` for successful dispatch and successful request completion
- `WARN` for expected caller or service failures such as validation, auth, or not found
- `ERROR` for unhandled or internal failures

Do not use ad hoc `println!` debugging in place of structured logs.

## Verification Requirements

A service is not considered online until observability is verified.

Minimum verification:

1. one successful action shows a dispatch event and downstream request events
2. one failing action shows a dispatch failure with a stable `kind`
3. the failing path preserves enough transport or response detail to diagnose the class of failure
4. logs do not expose secrets

Verification may use:

- unit tests for shared helpers
- mock-server tests for request behavior
- live read-only smoke tests against a real service when available

Destructive actions do not need live verification by default, but their intent and outcome logging must follow the same contract.

## Onboarding Gate

When bringing a new service online, observability is required before the service is complete.

That means the service must have:

- dispatch logging at every public surface it exposes
- shared `HttpClient` request logging for its outbound calls
- correct error kind mapping
- redaction compliance
- verification evidence that the request path is traceable end to end

If those conditions are missing, the service is not fully online even if the CLI, MCP, or HTTP action itself works.

## Example Shapes

Illustrative success fields:

```json
{
  "surface": "http",
  "service": "unifi",
  "action": "sites.list",
  "request_id": "req-123",
  "method": "GET",
  "path": "/proxy/network/integration/v1/sites",
  "host": "unifi.local",
  "status": 200,
  "elapsed_ms": 42
}
```

Illustrative failure fields:

```json
{
  "surface": "cli",
  "service": "unifi",
  "action": "sites.list",
  "method": "GET",
  "path": "/proxy/network/integration/v1/sites",
  "host": "unifi.local",
  "kind": "network_error",
  "message": "tls validation failed: self-signed certificate",
  "elapsed_ms": 311
}
```
