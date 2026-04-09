# Architecture

`lab` is a pluggable homelab CLI and MCP server implemented as a Rust workspace with a hard split between reusable service clients and user-facing surfaces.

## Core Shape

- One workspace
- Two crates
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

### `crates/lab`

`lab` is the product binary. It owns:

- CLI commands
- MCP server registration and dispatch
- TUI plugin manager
- config loading
- output rendering
- install/uninstall flows
- doctor and operator workflows

It should stay thin. Business logic belongs in `lab-apis`.

## Golden Rule

If behavior is shared by CLI and MCP, it belongs in `lab-apis`. The CLI and MCP layers are adapters, not logic owners.

That rule is structural, not aspirational:

- `lab-apis` has no `clap`, `rmcp`, or `ratatui`
- `lab` depends on `lab-apis` rather than duplicating service logic

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

Per-service layout in `lab`:

- `src/cli/<service>.rs`
- `src/mcp/services/<service>.rs`

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

Service identifiers should use service-local newtypes rather than raw integers everywhere. The goal is to prevent mixing:

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
2. Construct the correct client from `lab-apis`
3. Dispatch a service operation
4. Let `HttpClient` handle auth, retry, timeout, and error mapping
5. Return typed data to the caller surface
6. Render via CLI, MCP envelope, or TUI view

## Config Boundary

`lab-apis` never reads config files or ambient env on its own. Config loading lives in `lab`.

- secrets: `~/.lab/.env`
- preferences: `~/.config/lab/config.toml`

The binary resolves those inputs, then constructs clients explicitly.

## Service Model

Each service gets:

- one feature flag
- one service module in `lab-apis`
- one CLI entrypoint
- one MCP dispatch module
- one `PluginMeta`
- one health-check implementation

The one exception is [`EXTRACT.md`](./EXTRACT.md): it is not a remote API client, but it still follows the same structural model to keep the architecture uniform.
