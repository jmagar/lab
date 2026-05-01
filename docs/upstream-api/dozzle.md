# Dozzle Upstream API Notes

**Last reviewed:** 2026-05-01
**Upstream repository:** `https://github.com/amir20/dozzle`
**Reviewed commit:** `b3e5f2d2` (`docs: add default profile guide`)
**Reviewed image:** `amir20/dozzle:latest`, reporting `v10.5.1` from `/api/version`

Dozzle does not publish an OpenAPI document for its local web API. This file is
the source spec for Lab's Dozzle onboarding and is based on the upstream Go
routes in `/tmp/dozzle/internal/web/` plus a live local probe.

## Base Path

Dozzle supports `DOZZLE_BASE` for serving the UI and API below a path prefix.
Lab v1 does not model `DOZZLE_BASE` separately. Operators should include any
configured base path in `DOZZLE_URL`, for example:

```bash
DOZZLE_URL=http://dozzle.example.local/dozzle
```

## Authentication

Dozzle API routes under `/api` are authenticated when Dozzle is configured with
an auth provider. `/healthcheck` remains public.

Lab v1 supports:

| Lab env var | Dozzle behavior |
|---|---|
| `DOZZLE_URL` | Required base URL, including any `DOZZLE_BASE` prefix |
| `DOZZLE_SESSION_COOKIE` | Optional pre-provided `jwt=...` cookie value |

Lab v1 deliberately does not implement username/password login or forward-proxy
auth headers. Upstream simple auth creates an HttpOnly `jwt` cookie via
`POST /api/token` with form fields `username` and `password`; that flow is
deferred until a later auth-specific task.

## Security Boundary

Dozzle's Docker socket access is sensitive. Even a read-only socket can expose
container metadata and logs that contain secrets. Lab v1 therefore exposes only
read-only endpoints, never upstream actions or shell routes, and must bound log
and SSE reads by count, bytes, and timeout.

## V1 Routes

| Lab action | Method | Upstream route | Notes |
|---|---|---|---|
| `server.health` | GET | `/healthcheck` | Public. Returns HTTP 200 when at least one configured host is healthy; 500 otherwise. |
| `server.version` | GET | `/api/version` | Returns `text/html` body like `<pre>v10.5.1</pre>`; Lab strips the wrapper. |
| `containers.list` | GET | `/api/events/stream` | SSE stream. Lab reads only until the first `containers-changed` event or a timeout. |
| `logs.fetch` | GET | `/api/hosts/{host}/containers/{id}/logs` | JSONL by default. Requires stdout and/or stderr query flags. Lab applies max line/byte limits. |
| `logs.fetch-plain` | GET | `/api/hosts/{host}/containers/{id}/logs` | Same route with `Accept: text/plain`. Lab applies max byte limits. |

### Log Query Parameters

Upstream accepts these query keys for historical logs:

| Query key | Meaning | Lab v1 |
|---|---|---|
| `stdout` | Include stdout logs when present | Supported |
| `stderr` | Include stderr logs when present | Supported |
| `from` | RFC3339Nano start timestamp | Deferred unless implemented with tests |
| `to` | RFC3339Nano end timestamp | Deferred unless implemented with tests |
| `filter` | Regex filter | Deferred |
| `jsonOnly` | Only complex JSON log events | Deferred |
| `everything` | Fetch all available history | Set internally only; not exposed as a caller parameter |
| `min` | Ring buffer minimum | Deferred |
| `maxStart` | Stream start cap | Deferred |
| `lastSeenId` | Cursor-style ID | Deferred |
| `startId` | Cursor-style ID | Deferred |

Any Lab-supported caller parameter must be explicitly listed in the dispatch
catalog and covered by an exact query-string test. Lab sets upstream
`everything=1` internally for `logs.fetch` and `logs.fetch-plain` so operators
get historical logs from a bounded request; the user-facing caps remain
`max_lines`, `max_bytes`, and `timeout_ms`.

## Deferred Read Routes

These are read-like upstream routes but are not part of Lab v1:

| Method | Upstream route | Reason deferred |
|---|---|---|
| GET | `/api/hosts/{host}/containers/{id}/logs/stream` | Long-running SSE log stream needs a separate bounded streaming design. |
| GET | `/api/hosts/{host}/logs/stream` | Host-wide stream is broader than v1 container-focused log fetch. |
| GET | `/api/hosts/{host}/logs/mergedStream/{ids}` | Merged stream has path-encoded compound IDs and long-running stream behavior. |
| GET | `/api/containers/{hostIds}/download` | Download payload shape and size limits need separate design. |
| GET | `/api/labels/{labels}/logs/stream` | Label stream can expose broad multi-container logs. |
| GET | `/api/groups/{group}/logs/stream` | Group stream can expose broad multi-container logs. |
| GET | `/api/host-groups/{group}/logs/stream` | Group stream can expose broad multi-container logs. |
| GET | `/api/releases` | Product update information is not needed for log observation. |
| GET | `/api/cloud/status` | Cloud feature is outside local log observation. |
| GET | `/api/cloud/config` | Cloud feature is outside local log observation. |
| GET | `/api/profile/avatar` | Profile/UI data is outside service v1. |

## Excluded Mutating Routes

The following upstream routes are out of scope and must not be exposed by Lab v1.

| Method | Upstream route | Condition |
|---|---|---|
| POST | `/api/hosts/{host}/containers/{id}/actions/update` | Mounted only when `DOZZLE_ENABLE_ACTIONS` is true |
| POST | `/api/hosts/{host}/containers/{id}/actions/{action}` | Mounted only when `DOZZLE_ENABLE_ACTIONS` is true |
| PATCH | `/api/profile` | Always an authenticated profile mutation |
| POST | `/api/notifications/rules` | Notification mutation |
| PUT | `/api/notifications/rules/{id}` | Notification mutation |
| PATCH | `/api/notifications/rules/{id}` | Notification mutation |
| DELETE | `/api/notifications/rules/{id}` | Notification mutation |
| POST | `/api/notifications/dispatchers` | Notification mutation |
| PUT | `/api/notifications/dispatchers/{id}` | Notification mutation |
| DELETE | `/api/notifications/dispatchers/{id}` | Notification mutation |
| POST | `/api/notifications/preview` | Notification evaluation side effect |
| POST | `/api/notifications/test-webhook` | External notification side effect |
| PATCH | `/api/cloud/config` | Cloud config mutation |
| DELETE | `/api/cloud/config` | Cloud config mutation |
| POST | `/api/cloud/feedback` | Cloud side effect |
| POST | `/api/token` | Login creates a cookie; deferred auth flow |
| DELETE | `/api/token` | Logout clears a cookie; deferred auth flow |

## Excluded Shell Routes

| Method | Upstream route | Condition |
|---|---|---|
| GET | `/api/hosts/{host}/containers/{id}/attach` | Mounted only when `DOZZLE_ENABLE_SHELL` is true |
| GET | `/api/hosts/{host}/containers/{id}/exec` | Mounted only when `DOZZLE_ENABLE_SHELL` is true |

## Live Probe Summary

On 2026-05-01 a temporary container was started with a read-only Docker socket,
`DOZZLE_NO_ANALYTICS=1`, and no action/shell flags. The probe confirmed:

| Route | Result |
|---|---|
| `/healthcheck` | HTTP 200 |
| `/api/version` | `<pre>v10.5.1</pre>` |
| `/api/events/stream` | emitted `containers-changed` then stats/events |
| `/api/hosts/{host}/containers/{id}/logs?stdout=1&stderr=1&everything` | JSONL log rows |
