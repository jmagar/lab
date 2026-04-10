# api/ — axum HTTP surface

This directory is the **HTTP transport layer** for `lab`. It's a third peer to the CLI and MCP surfaces, built on **axum 0.8** + **tower-http**. It does not contain business logic — handlers are thin shims that call `lab-apis` clients, exactly like `cli/` and `mcp/`.

## Transport parity

The API mirrors the MCP action+subaction dispatch shape so clients can share logic across transports:

```
POST /v1/radarr
{ "action": "movie.search", "params": { "query": "The Matrix" } }
```

- **One route group per service**, mounted at `/v1/<service>`.
- Handlers dispatch on `action` exactly like `mcp/services/<service>.rs`.
- **Error envelopes are byte-identical to MCP envelopes.** Handlers return `Result<Json<T>, ToolError>` from `crate::dispatch::error` (or `crate::api::error` which re-exports it). `ToolError` implements `IntoResponse` — HTTP status is derived from `kind()`, never hand-assigned per-handler. Do **not** wrap `ToolError` in `ApiError::Internal`.
- Built-in per-service `GET /v1/<service>/actions` mirrors the `lab://<service>/actions` MCP resource. Use the shared `build_catalog()` — do not duplicate catalog logic.

## Files

| File | Purpose |
|------|---------|
| `api.rs` (parent) | Module declarations + re-exports. |
| `state.rs` | `AppState` — holds `lab-apis` clients, cloned per request (cheap `Arc` inside). |
| `error.rs` | `ApiError` + `ApiResult<T>` + `IntoResponse` mapping from `kind()` → HTTP status. |
| `router.rs` | `build_router(state)` — composes feature-gated routes + middleware stack. |
| `health.rs` | `GET /health` liveness + `GET /ready` readiness. |
| `services/<service>.rs` | Per-service route group (feature-gated). Thin dispatch shims. |

## Middleware stack

Applied in `router.rs`, top-to-bottom:

1. `SetRequestId` (UUID v4) — propagated as `x-request-id`.
2. `TraceLayer` — tracing spans per request with method, path, status, latency.
3. `TimeoutLayer` (30s default) — upstream service calls must honor their own shorter budgets.
4. `CompressionLayer` — gzip.
5. `CorsLayer` — permissive by default; tighten via config in production deployments.
6. `PropagateRequestId` — echoes `x-request-id` back in response.

Never add business-logic middleware here. Auth/rate-limit belong in their own layers, not in router setup.

## Status code mapping

`ApiError::status()` is the **only** place HTTP status codes are assigned. Handlers return `Result<Json<T>, ToolError>` and let the error type do the mapping:

| `kind()` | Status |
|----------|--------|
| `auth_failed` | 401 |
| `not_found` | 404 |
| `rate_limited` | 429 (+ `Retry-After` header when available) |
| `validation_failed`, `missing_param`, `invalid_param` | 422 |
| `unknown_action`, `unknown_instance` | 400 |
| `network_error`, `server_error` | 502 |
| `decode_error`, `internal_error` | 500 |

Do not return raw `StatusCode` from handlers. Always go through `ApiError`.

## Destructive actions

Actions marked `ActionSpec.destructive == true` require confirmation via:

- `"confirm": true` in the JSON request `params` object (boolean, not string).

Without this, the gate returns `400` with `kind: "confirmation_required"`. This is the HTTP equivalent of the MCP elicitation flow and the CLI `-y` flag.

The gate is enforced in `services/helpers.rs::handle_action()`.

**Security decision — `X-Lab-Confirm` header removed:** A header-based bypass (`X-Lab-Confirm: yes`) was removed because the API sits behind a reverse proxy that may forward arbitrary request headers by default (common Caddy/Traefik behavior). A misconfigured or compromised upstream can inject headers but cannot inject the JSON request body, making body params (`"confirm": true`) the only injection-safe confirmation signal. Do not re-add header-based confirmation without also requiring the reverse proxy to explicitly strip it.

## Feature gating

Per-service route modules under `services/` are `#[cfg(feature = "<service>")]`. The router builder conditionally mounts them:

```rust
#[cfg(feature = "radarr")]
let router = router.nest("/v1/radarr", services::radarr::routes(state.clone()));
```

Never hard-link service handlers from the top-level router — always conditional.

## Auth

The API itself is unauthenticated by default (homelab use case). Production deploys put it behind a reverse proxy (Caddy/Traefik/Tailscale serve) or add an auth layer via `tower::ServiceBuilder` in `router.rs`. Do not bake auth into handlers.

## What does NOT belong here

- **Business logic.** Belongs in `lab-apis/src/<service>/client.rs`.
- **`reqwest` calls.** Use the service client from `AppState`.
- **JSON shape definitions.** Use `lab-apis` types directly with `serde_json::Json(t)`.
- **Error types.** Wrap `SdkError` via `From` — don't define per-service HTTP errors.
- **Retries.** The SDK's `HttpClient` handles backoff; the API layer just surfaces outcomes.
