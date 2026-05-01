# Dozzle API Coverage

**Last updated:** 2026-05-01
**Source spec:** `docs/upstream-api/dozzle.md`
**SDK surface:** `crates/lab-apis/src/dozzle/client.rs` (5 public read-only methods)
**Shared dispatch:** `crates/lab/src/dispatch/dozzle/` (catalog.rs, dispatch.rs, client.rs, params.rs)
**MCP registration:** `crates/lab/src/registry.rs` (scaffolded direct registry entry)
**CLI surface:** `crates/lab/src/cli/dozzle.rs` (Tier 2 generic `action` + `KEY=VALUE` shim)
**API handler:** `crates/lab/src/api/services/dozzle.rs` (thin dispatch adapter)

## Legend

| Symbol | Meaning |
|---|---|
| [x] | Planned for Dozzle v1 |
| [ ] | Not implemented yet |
| [-] | Deliberately out of scope |

> **Auth note:** Dozzle has no API-token auth. Lab v1 supports unauthenticated
> Dozzle and an optional pre-provided `DOZZLE_SESSION_COOKIE`. Username/password
> login and forward-proxy auth are deferred.

## V1 Action Plan

| Action | Upstream route | SDK | Dispatch | MCP | CLI | API | Live |
|---|---|---|---|---|---|---|---|
| `server.health` | `GET /healthcheck` | [x] | [x] | [x] | [x] | [x] | [x] |
| `server.version` | `GET /api/version` | [x] | [x] | [x] | [x] | [x] | [x] |
| `containers.list` | bounded `GET /api/events/stream` | [x] | [x] | [x] | [x] | [x] | [x] |
| `logs.fetch` | bounded `GET /api/hosts/{host}/containers/{id}/logs` | [x] | [x] | [x] | [x] | [x] | [x] |
| `logs.fetch-plain` | bounded same route with `Accept: text/plain` | [x] | [x] | [x] | [x] | [x] | [x] |

## Required Environment

| Env var | Required | Secret | Notes |
|---|---|---|---|
| `DOZZLE_URL` | yes | no | Base URL, including any upstream `DOZZLE_BASE` prefix |
| `DOZZLE_SESSION_COOKIE` | no | yes | Pre-provided Dozzle `jwt=...` cookie value |

## Planned Parameters

### `server.health`

No parameters.

### `server.version`

No parameters.

### `containers.list`

| Param | Type | Required | Notes |
|---|---|---|---|
| `max_events` | integer | no | Conservative default and hard cap enforced in SDK |
| `timeout_ms` | integer | no | Conservative default and hard cap enforced in SDK |
| `max_bytes` | integer | no | Hard response cap for SSE parsing |

### `logs.fetch`

| Param | Type | Required | Notes |
|---|---|---|---|
| `host` | string | yes | Encoded as one path segment |
| `container_id` | string | yes | Encoded as one path segment |
| `stdout` | boolean | no | Include stdout; at least one of stdout/stderr must be true |
| `stderr` | boolean | no | Include stderr; at least one of stdout/stderr must be true |
| `max_lines` | integer | no | Hard line cap |
| `max_bytes` | integer | no | Hard byte cap |
| `timeout_ms` | integer | no | Hard request/read timeout |

Lab does not expose Dozzle's upstream `everything` switch as a user parameter.
The SDK sets `everything=1` internally so bounded historical fetches return
useful logs; Lab still enforces `max_lines`, `max_bytes`, and `timeout_ms`.

### `logs.fetch-plain`

Same parameters as `logs.fetch`, but returns bounded text plus truncation
metadata instead of parsed JSONL log events.

## Deferred

| Candidate | Reason |
|---|---|
| `events.snapshot` | Broader SSE snapshot is not needed for first read-only service value. |
| `logs.stream-sample` | Long-running SSE log stream needs a separate bounded streaming design. |
| Username/password login | Requires cookie extraction/refresh behavior and auth-specific tests. |
| Forward-proxy auth headers | Must be designed deliberately to avoid header spoofing/leakage. |
| Container actions | Mutating routes are mounted only with `DOZZLE_ENABLE_ACTIONS`; excluded from v1. |
| Shell attach/exec | Shell routes are mounted only with `DOZZLE_ENABLE_SHELL`; excluded from v1. |
| Notification/profile/cloud mutation | Outside read-only log observation. |

## Destructive Actions

None. Every Dozzle v1 action must be marked `destructive: false`, and the
dispatch tests must include a guard that fails if a Dozzle action is marked
destructive.

## Live Test Evidence

Initial investigation on 2026-05-01 used a temporary `amir20/dozzle:latest`
container with a read-only Docker socket, `DOZZLE_NO_ANALYTICS=1`, and no
`DOZZLE_ENABLE_ACTIONS` or `DOZZLE_ENABLE_SHELL`.

| Probe | Result |
|---|---|
| `GET /healthcheck` | HTTP 200 |
| `GET /api/version` | `<pre>v10.5.1</pre>` |
| `GET /api/events/stream` | emitted `containers-changed` and `container-stat` events |
| `GET /api/hosts/{host}/containers/{id}/logs?stdout=1&stderr=1&everything` | returned JSONL log rows |

Final smoke on the implemented Lab surfaces used the same temporary Dozzle
container and a single discovered container id.

| Surface | Command shape | Result |
|---|---|---|
| CLI | `lab --json dozzle server.health` | `reachable=true` |
| CLI | `lab --json dozzle server.version` | `version=v10.5.1` |
| CLI | `lab --json dozzle containers.list` | 1 container returned |
| CLI | `lab --json dozzle logs.fetch ... max_lines=20 max_bytes=1048576 timeout_ms=5000` | 3 events, `truncated=false` |
| CLI | `lab --json dozzle logs.fetch-plain ... max_lines=20 max_bytes=1048576 timeout_ms=5000` | 323 bytes, `truncated=false` |
| API | `POST /v1/dozzle {"action":"server.version"}` | `version=v10.5.1` |
| API | `POST /v1/dozzle {"action":"logs.fetch",...}` | 3 events, `truncated=false` |
| API | `POST /v1/dozzle {"action":"not.real"}` | HTTP 400, `kind=unknown_action` |
| MCP | `mcporter call ... dozzle action=server.version` | `ok=true`, `version=v10.5.1` |
| MCP | `mcporter call ... dozzle --args '{"action":"logs.fetch",...}'` | `ok=true`, 3 events, `truncated=false` |

## Implementation Test Evidence

Implementation checks run on 2026-05-01:

| Command | Result |
|---|---|
| `cargo test --manifest-path crates/lab-apis/Cargo.toml --features dozzle --test dozzle_client` | 8 passed |
| `cargo test --manifest-path crates/lab/Cargo.toml --features dozzle --lib dozzle::tests` | 7 passed |
| `cargo test --manifest-path crates/lab/Cargo.toml --features dozzle --lib api::services::dozzle::tests` | 2 passed |
| `cargo run --manifest-path crates/lab/Cargo.toml --all-features -- audit onboarding dozzle --json` | passed all checks |
| `cargo check --manifest-path crates/lab/Cargo.toml --all-features` | passed with existing unrelated warnings |

`cargo build --manifest-path crates/lab/Cargo.toml --no-default-features --features dozzle`
currently fails before Dozzle-specific code due existing feature-isolation gaps:
`crate::dispatch::deploy` is referenced from `node/update.rs` while the
`deploy` feature is off, and `gateway/manager.rs` references marketplace URL
validation gated behind `mcpregistry`.
