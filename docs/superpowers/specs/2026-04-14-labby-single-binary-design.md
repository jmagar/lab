# Labby Single-Binary Delivery Design

**Date:** 2026-04-14

## Goal

Deliver Labby as part of `lab serve` so one Rust binary can ship and run the CLI, MCP server, HTTP API, and web UI without requiring a Node runtime in production.

## Context

Today the repo is split in an awkward way:

- `lab serve` already owns the real control plane: dispatch, gateway management, auth, config, and MCP runtime state.
- `apps/gateway-admin` is a Next.js app that still carries a small server-side adapter layer to translate UI needs into the Rust gateway API.
- Production use currently requires both the Rust server and a Next server, even though the web product is fundamentally an operator console over the Rust backend.

That split weakens packaging, auth boundaries, debugging, and deployment ergonomics. The target state is a true single-backend architecture: Rust owns all privileged behavior, and the web app becomes a static client over the Rust API.

## Decision

Adopt a static-exported Labby frontend that is embedded into the `lab` binary and served directly by Axum from `lab serve`.

This is preferred over spawning `next start` because it gives a real single-binary deployment, removes the runtime Node dependency, and aligns the web UI with the existing backend ownership model.

## Non-Goals

This design does not include:

- web OAuth UX implementation
- gateway log viewer implementation
- redesign of the gateway domain model
- replacement of existing MCP/API auth semantics
- support for arbitrary Next.js server features in production

Those are follow-on product items and should remain out of scope for the initial architecture migration.

## Architecture

### System Boundary

`lab` becomes the only backend. It owns:

- API contracts and view models
- gateway config persistence
- upstream connection lifecycle
- auth and session behavior
- operator-visible diagnostics and error messages
- static web asset delivery

Labby becomes an unprivileged browser application that only:

- renders UI
- calls the Rust API
- subscribes to Rust-owned event/log streams
- handles client-side routing and local interaction state

No privileged logic, secret handling, or adapter behavior should remain in the Next server layer after the migration.

### Runtime Surface Layout

`lab serve` exposes one Axum router tree with reserved backend paths and a root-mounted SPA:

- `/v1/**` for HTTP API
- `/mcp` for MCP streamable HTTP
- `/health` and `/ready`
- OAuth and auth metadata routes already owned by `lab`
- `/<static-assets>` for hashed Labby assets
- `/` plus non-API browser paths for the Labby SPA shell

Unknown browser routes such as `/gateways`, `/gateways/<id>`, and `/docs` should resolve to the exported app shell. Unknown API/auth/MCP routes must keep normal backend semantics and must never fall through to the SPA.

### Delivery Model

The frontend is built ahead of time and consumed by the Rust binary. Production runtime must not require Node.

Preferred packaging model:

- export Labby as static assets during build/release
- embed the exported assets into the Rust binary
- serve embedded files from Axum with explicit cache policy

Fallback model if binary size becomes unacceptable:

- ship a deterministic sidecar asset directory with the binary

The initial design should assume embedding as the primary path because it best satisfies the one-binary product goal.

## Frontend Constraints

Labby must become static-export safe. In practical terms that means:

- no dependence on Next route handlers in production
- no server actions
- no Next-owned privileged session or token logic
- no runtime requirement for a Node-hosted Next server

This is not a UI compromise. A static SPA can still support:

- rich tables, forms, and setup flows
- live polling
- SSE or WebSocket log/event streams
- auth-aware screens
- detailed diagnostics and guided recovery messaging

The constraint is architectural, not visual: Rust owns backend behavior; the frontend consumes it.

## Backend Contract Changes

The current Next-side gateway adapter and normalization logic should move into Rust so the browser can call a first-class web API directly.

The API should expose UI-oriented view models where appropriate, rather than forcing the browser to reconstruct operator-facing state from low-level dispatch responses. That includes:

- normalized gateway list/detail payloads
- operator-readable status and warning text
- explicit probe/test results
- exact error messages and guidance
- stable route shapes for future event/log streaming

This keeps the frontend simple and makes later features like web OAuth and gateway log viewing fit naturally into the Rust control plane.

## Build and Release Design

### Build Pipeline

The build/release flow should produce frontend assets before the final Rust packaging step. The resulting contract is:

1. install frontend dependencies
2. build/export Labby static assets
3. feed those assets into the Rust build or packaging step
4. produce one distributable binary

The Rust build should fail clearly if the expected Labby asset manifest is missing when web embedding is enabled.

### Local Development

Development should keep the faster split mode:

- `next dev` for frontend iteration
- `lab serve` for backend/API iteration

Production and release packaging should use embedded static assets. This keeps the developer feedback loop fast without compromising the deployment target.

### Runtime Asset Policy

Axum should serve:

- hashed JS/CSS/assets with long-lived immutable caching
- `index.html` with no-store or short-lived caching
- correct content types and compression

The web asset layer must preserve existing backend middleware and must not accidentally weaken auth or observability boundaries.

## Error Handling and Observability

Because `lab serve` becomes the sole runtime host, it must be the authoritative source for web-serving diagnostics too.

Required behavior:

- startup errors clearly distinguish frontend asset packaging failures from API bind/auth failures
- logs state whether embedded assets are available and where the SPA is mounted
- requests to SPA assets remain observable without drowning out API/MCP dispatch logs
- client-visible failures for missing assets or malformed export state should be explicit and actionable

The frontend should continue surfacing exact, operator-readable error messages from Rust rather than generic browser failures.

## Migration Plan Shape

### Phase 1: Make Labby export-safe

- remove production dependence on Next route handlers
- move remaining server-only adapter behavior out of `apps/gateway-admin/lib/server/*`
- ensure browser routes and asset paths work under static export assumptions

### Phase 2: Move UI-specific backend shaping into Rust

- add stable web-facing API/view-model routes in `lab`
- preserve current UX quality for status, warnings, tests, and guidance
- keep the browser as a thin client over those Rust endpoints

### Phase 3: Serve Labby from Axum

- add embedded asset serving
- add SPA fallback routing
- reserve backend paths so API/auth/MCP semantics remain unchanged
- make `lab serve` the canonical production entrypoint for the full product

### Phase 4: Finalize release ergonomics

- wire frontend export into release/build automation
- document the local-dev vs production-runtime split
- verify a fresh binary can boot the full product with no Node runtime

## Verification

The architecture is complete only when all of the following are true:

- `lab serve` can boot with embedded Labby assets and serve the UI at `/`
- client-side routes load correctly via SPA fallback
- `/v1`, `/mcp`, health, and auth routes continue working without collision
- the web UI functions without any Next server process
- release artifacts require no Node runtime
- startup and request logs clearly describe both API and web serving behavior

## Risks

### Export drift

Labby may quietly reintroduce Next server-only features later. The mitigation is to make static export compatibility part of CI and release verification.

### Contract duplication

If Rust and the frontend each keep their own normalization logic, the migration will fail halfway. The mitigation is to make Rust the single owner of operator-facing backend view models.

### Binary size growth

Embedding assets increases binary size. That is acceptable initially because it preserves the product goal. If it becomes painful, sidecar assets remain a fallback, but should not drive the first design.

## Backlog Follow-Ons

These are intentionally deferred from the architecture migration:

- Web OAuth UX and session flows owned by `lab`
- Gateway log viewer backed by Rust API or event streaming

They should be tracked as backlog work, not pulled into the single-binary migration itself.
