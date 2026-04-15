# Architecture

`lab` is a pluggable homelab CLI and MCP server implemented as a Rust workspace with a split between reusable upstream-facing SDK clients and product-facing dispatch and surface layers.

## Core Shape

- One workspace
- Three crates
- One binary
- Many feature-gated services
- One MCP tool per service

## Crate Split

### `crates/lab-apis`

`lab-apis` is the pure SDK layer. It owns:

- typed service clients
- request and response models
- auth handling
- shared HTTP behavior
- shared error taxonomy
- shared action metadata
- plugin metadata
- health-check contracts

It does not own CLI parsing, MCP transport, TUI rendering, `.env` file loading, or shell-facing UX.

### `crates/lab-auth`

`lab-auth` is the auth middleware crate. It owns:

- OAuth 2.0 authorization server (Google OIDC provider)
- JWT signing and validation (RS256)
- SQLite-backed token and session storage
- axum middleware and route handlers

It is separated from `lab-apis` because it depends on `axum`, which is forbidden in the pure SDK crate. It does not own CLI parsing, MCP transport, or TUI rendering.

### `crates/lab`

`lab` is the product binary. It owns:

- CLI commands
- MCP server registration and dispatch
- TUI plugin manager
- config loading
- output rendering
- install/uninstall flows
- doctor and operator workflows

It must stay thin at the surface boundary, but it still owns shared product dispatch and product-local systems such as gateway and upstream management.

## Golden Rule

If behavior is shared across product surfaces, it belongs in one shared execution layer. Upstream API logic belongs in `lab-apis`; product-surface dispatch belongs in `crates/lab/src/dispatch`. The CLI and MCP layers are adapters, not logic owners.

That rule is structural, not aspirational:

- `lab-apis` has no `clap`, `rmcp`, `ratatui`, or `axum`
- `lab-auth` has no `clap`, `rmcp`, or `ratatui`
- `lab` depends on `lab-apis` and `lab-auth` rather than duplicating service logic

## Module Layout

The workspace uses modern Rust module layout:

- no `mod.rs`
- a module `foo` is declared in `foo.rs`
- its submodules live in `foo/`

Per-service layout in `lab-apis`:

- `<service>.rs`
- `<service>/client.rs`
- `<service>/types.rs`
- `<service>/error.rs`

Per-service layout in `lab` typically includes:

- `src/dispatch/<service>.rs` plus `src/dispatch/<service>/`
- `src/cli/<service>.rs`
- `src/mcp/services/<service>.rs`
- `src/api/services/<service>.rs` when the service is exposed over HTTP

## Shared Contracts

The architecture is anchored around a few cross-cutting contracts:

- `ServiceClient`: common health-check interface
- `ServiceStatus`: normalized health result
- service-specific ID newtypes
- `Auth`: shared auth model
- `ApiError`: normalized transport-layer error taxonomy
- `HttpClient`: shared request/retry/logging/error-mapping layer
- `ActionSpec` / `ParamSpec`: service action catalog schema
- `PluginMeta`: service metadata for install/TUI/doctor flows

These contracts keep service modules consistent and make CLI, MCP, TUI, and operator tooling compose cleanly.

### `ServiceClient`

Every service client implements a common health surface:

- `name()`
- `service_type()`
- `health()`

That gives `lab health`, `lab doctor`, TUI status views, and MCP `status` surfaces a shared model without forcing all other service operations into one trait.

### `ServiceStatus`

`ServiceStatus` is the normalized health result shape. Its important fields are:

- reachability
- auth state
- optional version
- latency
- optional detail message

Rules:

- unreachable implies auth is not OK
- health probes have a shorter timeout budget than ordinary requests
- transport failures become structured status data rather than panics

### ID Newtypes

Service identifiers must use service-local newtypes rather than raw integers everywhere. The goal is to prevent mixing:

- internal ids
- external provider ids
- ids from different services

## Runtime Surfaces

The same service logic is exposed through three product surfaces:

- CLI: `lab <service> <command>`
- MCP: `lab serve`
- TUI: `lab plugins`

All three consume the same service metadata and service clients.

The canonical ownership and dependency rules between `lab-apis`, the shared dispatch layer, and the product surfaces live in [DISPATCH.md](./DISPATCH.md).

## Logging Shape

Observability is a mandatory shared contract, not a per-service convention.

The canonical source of truth is [OBSERVABILITY.md](./OBSERVABILITY.md).

High-level ownership is:

- `lab` owns caller context and dispatch logging
- `lab-apis::core::HttpClient` owns outbound request logging and transport failure detail

Required boundary rules:

- CLI, MCP, and HTTP must emit one dispatch event per user-visible action
- `HttpClient` must emit `request.start` plus `request.finish` or `request.error` for every outbound call
- health probes must be distinguishable from normal actions
- destructive actions must log intent and outcome

Field-level requirements, redaction rules, and verification gates live in [OBSERVABILITY.md](./OBSERVABILITY.md). Do not redefine them piecemeal in service modules.

## Data Flow

Normal request flow:

1. Load config in `lab`
2. Construct the correct SDK client or product-local subsystem
3. Dispatch through the shared `crates/lab/src/dispatch` layer
4. Let `HttpClient` handle auth, retry, timeout, and error mapping for upstream-backed services
5. Return typed or surface-neutral data to the caller surface
6. Render via CLI, MCP envelope, API envelope, or TUI view

## Config Boundary

`lab-apis` never reads config files or ambient env on its own. Config loading lives in `lab`.

- secrets: `~/.lab/.env`
- preferences: `config.toml` (`./` → `~/.lab/` → `~/.config/lab/`)

The binary resolves those inputs, then constructs clients explicitly.

## Service Model

Each service gets:

- one feature flag
- one service module in `lab-apis` for upstream-backed integrations
- one shared dispatch entry in `crates/lab/src/dispatch`
- one CLI entrypoint
- one MCP dispatch entry
- one `PluginMeta` when it participates in install/TUI/doctor flows
- one health-check implementation when it models a remotely configured service

There are product-local exceptions. [`EXTRACT.md`](./EXTRACT.md) is a synthetic bootstrap service, and [`GATEWAY.md`](./GATEWAY.md) is a product-local management surface for runtime upstream configuration rather than a feature-gated `lab-apis` integration.
