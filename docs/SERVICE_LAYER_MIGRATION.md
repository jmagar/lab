# Service Layer Migration

This guide describes how to migrate `lab` from the current surface-coupled shape to the target dispatch-layer architecture defined in [DISPATCH.md](./DISPATCH.md).

It is both:

- a migration guide
- a working checklist

## Goal

Move shared operation orchestration out of sibling surfaces and into a shared layer:

- from ad hoc logic in `cli/`, `mcp/services/`, and `api/services/`
- to `crates/lab/src/services/`

Target dependency direction:

- `cli -> services -> lab-apis`
- `mcp -> services -> lab-apis`
- `api -> services -> lab-apis`

Forbidden target state:

- `cli -> mcp`
- `api -> mcp`

## What Is Being Migrated

The migration is about shared product-surface orchestration, not upstream SDK logic.

Move into `services`:

- operation catalog
- param metadata
- param validation
- destructive-op metadata
- client and instance resolution
- SDK calls
- surface-neutral results
- surface-neutral dispatch errors

Do not move:

- `clap` parsing
- MCP envelopes
- HTTP status-code mapping
- axum routing
- CLI table rendering
- upstream request and response parsing from `lab-apis`

## Target Layout

```text
crates/lab/src/
  services.rs
  services/
    context.rs
    errors.rs
    params.rs
    radarr.rs
    bytestash.rs
    unifi.rs
```

Surface layout remains:

- `cli/` for typed CLI
- `mcp/` for MCP transport
- `api/` for HTTP API transport

## Migration Principles

1. Move one concern at a time.
2. Keep behavior stable while changing ownership.
3. Prefer helper extraction before broad rewrites.
4. Do not rewrite service SDK logic unless a bug requires it.
5. Keep MCP and HTTP API as thin wrappers over the same shared dispatch behavior.

## Recommended Phase Order

1. Shared helper extraction
2. Shared `services` layer introduction
3. Migrate ByteStash
4. Migrate UniFi
5. Migrate Radarr
6. Add verification tooling
7. Update coverage docs and onboarding docs as needed

## Phase 1: Extract Shared Helpers

Purpose:

- remove obvious duplication before moving larger behavior
- establish shared patterns inside `crates/lab/src`

### 1.1 Action-style CLI param parser

Extract shared helpers for:

- `Vec<String>` to `serde_json::Value`
- `key=value` parsing
- basic scalar coercion

Source duplicates today:

- `crates/lab/src/cli/bytestash.rs`
- `crates/lab/src/cli/unifi.rs`

Suggested target:

- `crates/lab/src/services/params.rs`
- or `crates/lab/src/cli/helpers.rs` if it remains CLI-only

### 1.2 Shared HTTP API dispatch wrapper

Extract shared HTTP API helper for:

- timing
- dispatch logging
- request handling boilerplate
- response wrapping

Source duplicates today:

- `crates/lab/src/api/services/radarr.rs`
- `crates/lab/src/api/services/bytestash.rs`
- `crates/lab/src/api/services/unifi.rs`

### 1.3 Shared client resolver skeleton

Create a shared location for:

- env lookup
- default-instance lookup
- named-instance lookup
- auth construction
- client construction

This can start small and service-specific, then converge.

## Phase 2: Introduce `services/`

Create:

- `crates/lab/src/services.rs`
- `crates/lab/src/services/context.rs`
- `crates/lab/src/services/params.rs`

Define the minimum shared types:

- `DispatchContext` — **minimal**: `surface: &'static str` + `instance: Option<String>` only. Do not add `request_id` or `operation` until a second service migrates and the pattern is proven. `request_id` requires axum middleware plumbing and is absent on CLI/MCP paths.
- **No `DispatchError`** — service dispatch functions return `Result<Value, ToolError>` directly. A separate `DispatchError` type adds a mapping layer with a catch-all arm trap and no structural benefit since both `services/` and surface adapters live in the same `lab` crate. `ToolError` already has the complete vocabulary (`UnknownAction`, `MissingParam`, `InvalidParam`, `UnknownInstance`, `Sdk`). See `docs/DISPATCH.md` Error Contract for the full rationale.
- `services/errors.rs` is therefore **not created**.

Decide initial result shape:

- use `serde_json::Value` initially to reduce migration cost

That is acceptable as long as the layer remains surface-neutral.

## Phase 3: Migrate ByteStash First

ByteStash is the easiest migration candidate because its CLI already mirrors the MCP action model.

### 3.1 Create `services/bytestash.rs`

Move from MCP-owned orchestration to shared orchestration:

- action matching
- param validation
- client resolution
- SDK calls

### 3.2 Make MCP wrap `services::bytestash`

MCP should keep:

- tool registration
- envelope conversion
- `help` and `schema`

MCP should stop owning the shared operation implementation.

### 3.3 Make CLI wrap `services::bytestash`

Short term:

- existing action-style CLI may call the shared service layer

Long term:

- replace with typed CLI if that becomes the repo-wide CLI contract

### 3.4 Make HTTP API wrap `services::bytestash`

The HTTP API handler should call the shared service layer, not the MCP dispatcher.

### 3.5 Verify

- unit tests pass
- MCP behavior unchanged
- HTTP API behavior unchanged
- CLI behavior unchanged
- observability fields still appear as expected

## Phase 4: Migrate UniFi

Repeat the ByteStash pattern for UniFi.

Special attention:

- large action catalog
- repeated param helpers
- instance and auth resolution

UniFi is a good test of whether the `services` layer scales to broad machine-facing services.

## Phase 5: Migrate Radarr

Radarr should also move to `services`, even if its human CLI remains typed.

### 5.1 Create `services/radarr.rs`

This shared layer should own:

- operation metadata
- client resolution
- common execution path for machine-facing actions

### 5.2 Keep typed CLI as a separate concern

The typed CLI remains a UX choice, not a reason to bypass the shared layer.

Possible mapping:

- `lab radarr movie get --id 123`
- maps to shared operation `movie.get`

### 5.3 Make MCP and HTTP API wrap `services::radarr`

They should no longer own the shared dispatch implementation.

## Phase 6: Standardize Typed CLI

If typed CLI is the repo contract, formalize and apply it.

Work items:

- define the typed CLI rule in onboarding and root guidance
- convert ByteStash and UniFi away from free-form `action + key=value` CLI if desired
- keep MCP and HTTP API machine-oriented

This phase is a product-UX choice layered on top of the architecture migration.

## Phase 7: Add Verification Tooling

Add a command or helper such as:

```bash
just service-check <name>
```

It should verify:

- feature flags in both crates
- service registration in CLI, MCP, and HTTP API
- coverage doc exists
- required canonical docs exist
- service compiles under its feature

Also consider:

- shared contract tests for dispatch errors
- shared contract tests for envelope shape
- shared contract tests for observability shape

## Migration Checklist

Use this checklist while executing the migration.

### Foundation

- [ ] Create `crates/lab/src/services.rs`
- [ ] Create `crates/lab/src/services/`
- [ ] Add `context.rs` and `params.rs` (no `errors.rs` — see Phase 2 note)
- [ ] Define `DispatchContext` (minimal: `surface` + `instance` only)
- [ ] Confirm result shape: `Result<Value, ToolError>` directly

### Shared Helpers

- [ ] Extract shared `key=value` CLI param parsing into `services/params.rs`
- [ ] Extract shared HTTP API dispatch wrapper with destructive confirmation gate
- [ ] Add shared client resolver skeleton (initially per-service, converge later)
- [ ] Add shared observability helper where useful

### ByteStash

- [ ] Create `crates/lab/src/services/bytestash.rs`
- [ ] Move operation matching and validation there
- [ ] Make MCP wrap `services::bytestash`
- [ ] Make CLI wrap `services::bytestash`
- [ ] Make HTTP API wrap `services::bytestash`
- [ ] Verify behavior stays stable

### UniFi

- [ ] Create `crates/lab/src/services/unifi.rs`
- [ ] Move operation matching and validation there
- [ ] Make MCP wrap `services::unifi`
- [ ] Make CLI wrap `services::unifi`
- [ ] Make HTTP API wrap `services::unifi`
- [ ] Verify behavior stays stable

### Radarr

- [ ] Create `crates/lab/src/services/radarr.rs`
- [ ] Move shared operation execution there
- [ ] Make MCP wrap `services::radarr`
- [ ] Make HTTP API wrap `services::radarr`
- [ ] Make typed CLI map onto `services::radarr`
- [ ] Verify behavior stays stable

### Typed CLI Standardization

- [ ] Decide whether ByteStash gets a typed CLI now or later
- [ ] Decide whether UniFi gets a typed CLI now or later
- [ ] Update docs to make typed CLI the default contract

### Verification Tooling

- [ ] Add `just service-check <name>` or equivalent
- [ ] Add contract tests for dispatch errors
- [ ] Add contract tests for envelope shape
- [ ] Add contract tests for observability shape

### Docs

- [ ] Update `docs/DISPATCH.md` if implementation details change
- [ ] Update `docs/SERVICE_ONBOARDING.md` with the final migration target
- [ ] Update `docs/coverage/*.md` for migrated services
- [ ] Update root and nearest `CLAUDE.md` files if needed

## Risks To Watch

- moving too much transport-specific code into `services`
- turning `services` into a second SDK layer instead of a shared product dispatch layer
- letting MCP-only envelope logic leak into `services`
- preserving old `cli -> mcp` or `api -> mcp` shortcuts during migration
- changing CLI UX and architecture at the same time without isolating the behavior changes
- **`cli/serve.rs` fan-out:** `serve.rs` references every service's MCP dispatcher. After each service migrates, verify `serve.rs` does not retain a direct `mcp::services::<service>::dispatch` call. The MCP module keeps a public forwarding shim during the migration window; remove it only after `serve.rs` is updated.
- **ACTIONS catalog drift:** `ACTIONS` must live in `services/<service>.rs` as the single source. MCP re-exports from there. Never copy the array — two lists drift and elicitation silently skips newly-added destructive actions.
- **Credential logging in dispatch wrapper:** The `handle_action` wrapper logs `action` and `elapsed_ms`. Never add `params` to that log event — `auth.login` and `auth.register` pass plaintext credentials through `params`. This prohibition must be enforced at code review, not by convention alone.
- **Per-request client construction:** `client_from_env` reads env vars and constructs a `reqwest::Client` (connection pool) on every dispatch call. Acceptable for the initial migration. Must be moved to `AppState` before any service with sustained HTTP API traffic migrates.

## Definition Of Done

The migration is complete when:

- no service CLI depends on MCP dispatch modules
- no HTTP API service depends on MCP dispatch modules
- shared operation semantics live in `crates/lab/src/services`
- typed CLI, MCP, and HTTP API all use the same shared execution path
- client and instance resolution are not duplicated across sibling surfaces
- docs reflect the final contract

## Suggested First Execution Slice

If executing this incrementally, start here:

1. extract shared CLI param parsing
2. add the shared HTTP API dispatch wrapper
3. create `services/bytestash.rs`
4. migrate ByteStash end to end

That gives one full vertical proof before larger migrations.

## Related Docs

- [DISPATCH.md](./DISPATCH.md)
- [SERVICE_ONBOARDING.md](./SERVICE_ONBOARDING.md)
- [OBSERVABILITY.md](./OBSERVABILITY.md)
- [ERRORS.md](./ERRORS.md)
- [SERIALIZATION.md](./SERIALIZATION.md)
- [reports/2026-04-08-service-onboarding-review.md](/home/jmagar/workspace/lab/docs/reports/2026-04-08-service-onboarding-review.md)
