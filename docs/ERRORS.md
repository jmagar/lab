# Errors

This document is the canonical error-handling contract for `lab`.

It defines:

- the shared transport error taxonomy
- the dispatcher-level error vocabulary
- the required envelope shapes for MCP and HTTP
- status-code mapping expectations
- when changing error kinds is a spec change

## Goal

Errors must be:

- stable across services
- machine-readable across transports
- structured enough for agents and operators to react programmatically
- specific enough to diagnose the failure class without inventing per-service vocabularies

## Ownership

Error handling is split across layers:

- `lab-apis` owns the canonical shared transport taxonomy via `ApiError`
- service modules may wrap `ApiError` with service-specific errors
- `lab` dispatch layers add caller and validation errors on top
- MCP and HTTP must emit stable structured envelopes derived from those kinds

## Canonical SDK Taxonomy

The shared transport taxonomy lives in `lab-apis::core::ApiError`.

Stable `kind()` values are:

- `auth_failed`
- `not_found`
- `rate_limited`
- `validation_failed`
- `network_error`
- `server_error`
- `decode_error`
- `internal_error`

These kinds are consumed by MCP and HTTP callers. Changing them is a spec change.

## Dispatcher-Level Kinds

Dispatch layers may add the following kinds on top of SDK errors:

- `unknown_action`
- `unknown_subaction`
- `missing_param`
- `invalid_param`
- `unknown_instance`

Additional MCP-only flow-control cases may include:

- `elicitation_declined`
- `elicitation_unsupported`

### HTTP-Only Dispatcher Kinds

The following kinds are emitted exclusively by the HTTP surface. MCP handles the same guard differently (via elicitation), and CLI handles it via `--yes` / `-y`.

#### `confirmation_required`

**When:** A destructive action (`ActionSpec.destructive == true`) is dispatched over HTTP without `params["confirm"] == true`.

**Surface:** HTTP only. MCP uses elicitation; CLI requires `--yes`.

**Resolution:** Set `"confirm": true` inside the request body's `params` object and re-submit.

**Status code:** `422 Unprocessable Entity`

**Envelope:**

```json
{ "kind": "confirmation_required", "message": "action `snippets.delete` is destructive — set `confirm: true` in params to proceed" }
```

**Implementation note:** Emitted as `ToolError::Sdk { sdk_kind: "confirmation_required" }` from `handle_action` in `crates/lab/src/api/services/helpers.rs`.

### MCP-Only Dispatcher Kinds

#### `upstream_error`

**When:** A proxied upstream MCP server call fails — connection lost, timeout, response too large (`LAB_UPSTREAM_MAX_RESPONSE_BYTES`, default 10 MB), or the upstream returned an error.

**Surface:** MCP only. Upstream proxy is MCP-transport infrastructure.

**Resolution:** Check upstream server health. Review circuit breaker status via `lab.help` or logs. If the upstream is consistently failing, it will be excluded from tool listings after 3 consecutive failures.

**Status code:** `502 Bad Gateway` (when mapped to HTTP, e.g. in error.rs)

**Envelope:**

```json
{ "kind": "upstream_error", "message": "upstream `my-server` call failed: connection refused" }
```

Do not invent new kinds casually. If a new cross-service kind is needed, update the owning docs and all public surfaces together.

### Upstream OAuth Kinds

The upstream OAuth (outbound) surface adds five stable kinds for operator- and user-facing failures in the authorization-code + PKCE flow against OAuth-protected upstream MCP servers. Full flow documented in [UPSTREAM.md](./UPSTREAM.md).

#### `oauth_needs_reauth`

**When:** The persisted upstream OAuth credential can no longer be used to obtain a valid access token, and the user must re-initiate authorization. Concrete triggers:

- the authorization server returned `invalid_grant` on refresh (refresh token revoked, rotated twice, or otherwise invalidated)
- the encrypted `token_blob` failed to decrypt (for example after `LAB_OAUTH_ENCRYPTION_KEY` rotation)
- a 401 was received on a non-idempotent request and retry is not safe
- no persisted credential exists yet for the `(upstream, subject)` pair

**Surface:** MCP proxied calls, `/mcp`, hosted UI, `/v1/gateway/oauth/status`.

**Resolution:** Start a fresh authorization via `POST /v1/gateway/oauth/start`.

**Status code:** `401 Unauthorized`.

#### `oauth_state_invalid`

**When:** The callback at `/auth/upstream/callback` cannot match the `state` parameter to a live pending-state row for the authenticated subject and requested upstream. Causes: missing session, replayed `state`, expired state (>10 min), cross-subject attempt, or cross-upstream-name attempt.

**Surface:** `/auth/upstream/callback` only.

**Resolution:** Re-initiate authorization.

**Status code:** `400 Bad Request`.

#### `oauth_resource_mismatch`

**When:** The authorization server refused the RFC 8707 `resource` parameter, or the returned access token's `aud` claim does not match the canonical upstream MCP URL.

**Surface:** Upstream OAuth manager (begin / callback / build_auth_client).

**Resolution:** Operator must verify the upstream MCP server URL in config and the AS registration match.

**Status code:** `502 Bad Gateway`.

#### `oauth_issuer_mismatch`

**When:** The AS metadata `issuer` is missing, or an endpoint origin (scheme + host + port) does not match the issuer origin (RFC 8414 §3.3 requirement).

**Surface:** Upstream OAuth manager (discovery).

**Resolution:** Operator must contact the upstream AS owner; this is an RFC 8414 §3.3 violation.

**Status code:** `502 Bad Gateway`.

#### `oauth_unsupported_method`

**When:** The upstream AS metadata omits `code_challenge_methods_supported` or advertises only `plain`. `lab` refuses to fall back from S256.

**Surface:** Upstream OAuth manager (discovery).

**Resolution:** The upstream AS must advertise `S256`. No workaround.

**Status code:** `502 Bad Gateway`.

## Wrapping Rules

Service-specific errors must:

- wrap `ApiError` transparently where possible
- preserve the underlying `kind()` semantics for transport-layer failures
- avoid forking the shared taxonomy into service-local equivalents

Public surface code must not stringify and discard the error kind.

### `From<ServiceError> for ToolError` Placement

All `From<XError> for ToolError` impls live in `crates/lab/src/dispatch/error.rs`,
feature-gated to their service. This ensures both MCP and HTTP surfaces share a
single conversion path. Do not place these impls in `mcp/services/` or
`api/services/` — that traps the conversion in one surface module.

Pattern:

```rust
#[cfg(feature = "foo")]
impl From<lab_apis::foo::error::FooError> for ToolError {
    fn from(e: lab_apis::foo::error::FooError) -> Self {
        let kind = match &e {
            FooError::Api(api) => api.kind(),
            FooError::NotFound { .. } => "not_found",  // service-specific variants
        };
        Self::Sdk {
            sdk_kind: kind.to_string(),
            message: e.to_string(),
        }
    }
}
```

## MCP Contract

MCP error responses must be structured and machine-readable.

Canonical MCP error envelope:

```json
{
  "ok": false,
  "service": "radarr",
  "action": "movie.add",
  "error": {
    "kind": "missing_param",
    "message": "missing parameter: root_folder"
  }
}
```

Rules:

- `kind` is the stable semantic tag
- `message` is human-readable diagnostic text
- additional structured keys such as `param`, `valid`, or `hint` may be included where relevant
- clients must not need to parse free-form prose to classify the error

## HTTP Contract

HTTP error responses must use the same semantic `kind` vocabulary as MCP.

Canonical HTTP error envelope:

```json
{
  "kind": "auth_failed",
  "message": "authentication failed"
}
```

Rules:

- HTTP and MCP must agree on the semantic kind
- HTTP may use transport-appropriate status codes, but the JSON body remains structured
- HTTP must not invent a second vocabulary for the same failure class

## HTTP Status Mapping

Default mapping expectations:

- `auth_failed` -> `401 Unauthorized`
- `not_found` -> `404 Not Found`
- `rate_limited` -> `429 Too Many Requests`
- `validation_failed` -> `422 Unprocessable Entity`
- `missing_param` -> `422 Unprocessable Entity`
- `invalid_param` -> `422 Unprocessable Entity`
- `unknown_action` -> `400 Bad Request`
- `unknown_instance` -> `400 Bad Request`
- `confirmation_required` -> `422 Unprocessable Entity`
- `network_error` -> `502 Bad Gateway`
- `server_error` -> `502 Bad Gateway`
- `upstream_error` -> `502 Bad Gateway`
- `oauth_needs_reauth` -> `401 Unauthorized`
- `oauth_state_invalid` -> `400 Bad Request`
- `oauth_resource_mismatch` -> `502 Bad Gateway`
- `oauth_issuer_mismatch` -> `502 Bad Gateway`
- `oauth_unsupported_method` -> `502 Bad Gateway`
- `internal_error` -> `500 Internal Server Error`

## Deploy Service Kinds

The `deploy` service (feature-gated) adds the following stable kinds, all
surfaced via `DeployError::kind()` in `lab-apis/src/deploy/error.rs`:

| `kind` | HTTP status | Meaning |
|--------|-------------|---------|
| `validation_failed` | 422 | Bad input (host alias, remote_path allowlist, etc.). Shares the standard `validation_failed → 422` mapping. |
| `ssh_unreachable` | 502 | SSH connection or auth failed for a target. |
| `build_failed` | 502 | Local `cargo build --release -p lab` failed. |
| `preflight_failed` | 502 | Remote arch probe, writable-dir check, or sha256 probe failed. |
| `transfer_failed` | 502 | Streaming the artifact to the remote failed. |
| `install_failed` | 502 | Atomic rename/backup on the remote failed. |
| `restart_failed` | 502 | `systemctl restart` or `is-active --wait` failed. |
| `verify_failed` | 502 | Post-install `lab --version` probe failed. |
| `partial_failure` | — | Multi-host run where some hosts failed; returned as HTTP 200 with `ok=false` in the body, not as an error response. |
| `conflict` | 409 | Another deploy is in progress for the same host. |
| `arch_mismatch` | 502 | Remote `uname -m` differs from local build triple. |
| `integrity_mismatch` | 502 | Remote sha256 of staged artifact differs from local. |
| `auth_failed` | 401 | `LAB_DEPLOY_TOKEN` missing or headless `confirm: true` rejected. Shares the standard `auth_failed → 401` mapping. |

The deploy-specific kinds (`ssh_unreachable`, `build_failed`, `preflight_failed`,
`transfer_failed`, `install_failed`, `restart_failed`, `verify_failed`,
`arch_mismatch`, `integrity_mismatch`, `conflict`) are registered in
`api/error.rs::IntoResponse` so they map to the correct HTTP status codes
when the deploy HTTP surface is wired.

MCP envelopes carry the redacted message from `DeployError::redacted_message()`;
the full structured detail is logged at WARN locally.

## Device Runtime Notes

The device runtime uses the same shared taxonomy.

Important cases in this implementation:

- master-only fleet query routes on a non-master device return `not_found`
- invalid OAuth relay target input returns `invalid_param`
- missing fleet store wiring returns `internal_error`
- failed master-bound HTTP uploads map through the normal transport-layer kinds rather than inventing device-local variants

## Message Rules

Messages must help diagnose the issue without changing the stable kind.

Rules:

- keep `kind` stable and small
- put diagnostic detail in `message`
- preserve enough detail to distinguish likely transport classes inside `network_error`
- do not leak secrets, tokens, cookies, or auth headers in messages

Examples of acceptable `network_error` message detail:

- DNS resolution failure
- TCP connect refused
- TLS validation failure
- timeout

## Spec-Change Rules

The following are spec changes:

- adding a new `ApiError::kind()` value
- renaming an existing `kind`
- changing MCP or HTTP envelope structure in a breaking way
- changing the expected status-code mapping for an existing kind

When making one of those changes, update:

- `docs/ERRORS.md`
- `docs/MCP.md`
- `docs/CONVENTIONS.md`
- `CLAUDE.md`
- any affected surface code and tests

## Verification Requirements

At minimum, verify:

1. SDK errors preserve the expected `kind()`
2. MCP emits the expected structured error envelope
3. HTTP emits the expected structured JSON error with the matching semantic kind
4. messages do not leak secrets

## Related Docs

- [CONVENTIONS.md](./CONVENTIONS.md)
- [MCP.md](./MCP.md)
- [CLI.md](./CLI.md)
- [OBSERVABILITY.md](./OBSERVABILITY.md)
