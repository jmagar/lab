# Service Onboarding

This is the canonical end-to-end workflow for bringing a new service online in `lab`.

Use this when adding a brand-new integration or when completing a partially implemented service.

## Goal

Bringing a service online means all of the following are true:

- the `lab-apis` client exists and owns the service logic
- the service is feature-gated in both crates
- the CLI shim exists
- the MCP dispatcher exists
- the HTTP API dispatcher exists
- the service is registered in the MCP registry, CLI router, HTTP router, and TUI metadata
- the service has a coverage doc under `docs/coverage/`
- the request path is observable end to end under the shared observability contract
- the implementation is tested at the unit level and verified against a live instance when possible

The service is not considered done if only one surface works.

## Source Of Truth

Start from the source spec that already lives in `docs/upstream-api/`:

- OpenAPI file for structured APIs
- hand-scraped markdown for vendor docs and flat RPC APIs

If the source document is missing or stale, refresh it first and then generate or update the coverage doc under `docs/coverage/`.

The coverage doc is planning and verification support. The source spec remains the contract.

For non-HTTP capability modules, the "source spec" may instead be:

- a local file format contract
- an SSH-backed workflow contract
- a shell or tool invocation contract
- an internal synthetic capability contract documented in `docs/`

`extract` is the reference example. The capability contract still needs to be documented before implementation starts.

## Working Rules

- business logic belongs in `crates/lab-apis/src/<service>/client.rs`
- CLI, MCP, and HTTP are thin adapters over the shared dispatch layer
- `lab-apis` must not read config files or ambient env directly
- every service is feature-gated
- destructive operations must be marked at the action level and honored consistently across surfaces
- one MCP tool per service, with `action`-based dispatch

The canonical adapter and dependency-direction rules live in [DISPATCH.md](./DISPATCH.md).

If you are writing logic in `crates/lab/src/cli/<service>.rs`, `crates/lab/src/mcp/services/<service>.rs`, or `crates/lab/src/api/services/<service>.rs`, the logic probably belongs in `lab-apis` instead.

## File Layout

### `lab-apis`

Use the standard module shape:

- `crates/lab-apis/src/<service>.rs`
- `crates/lab-apis/src/<service>/client.rs`
- `crates/lab-apis/src/<service>/types.rs`
- `crates/lab-apis/src/<service>/error.rs`

If the service needs shared primitives, add them in the service module tree rather than smearing them across CLI or transport code.

### `lab`

Surface code lives in:

- `crates/lab/src/services/<service>.rs`
- `crates/lab/src/cli/<service>.rs`
- `crates/lab/src/mcp/services/<service>.rs`
- `crates/lab/src/api/services/<service>.rs`

Registry and metadata wiring live in:

- `crates/lab/src/cli.rs`
- `crates/lab/src/mcp/registry.rs`
- `crates/lab/src/api/services.rs`
- `crates/lab/src/tui/metadata.rs`

Feature gating and re-exports live in:

- `crates/lab-apis/src/lib.rs`
- `crates/lab/Cargo.toml`

## Recommended Order

Bring a service online in this order:

1. verify or add the upstream API spec
2. create the `lab-apis` module
3. define types and the service error
4. implement the client methods
5. add observability
6. implement health
7. add feature flags and module re-exports
8. wire the CLI shim
9. wire the MCP dispatcher
10. wire the HTTP dispatcher
11. register metadata and runtime discovery
12. add and update docs
13. run tests and live verification

That order keeps the service boundary stable before the public surfaces are wired.

## Step 1: Spec And Coverage Doc

Start by deciding which source document owns the API:

- OpenAPI spec in `docs/upstream-api/<service>.openapi.json|yaml`
- hand-written markdown in `docs/upstream-api/<service>.md`

Then create or update the coverage doc in `docs/coverage/<service>.md`.

Coverage docs must answer:

- what the upstream surface contains
- what `lab-apis` methods exist
- what CLI commands exist
- what MCP actions exist
- what the HTTP handler exposes
- what is still missing

For openapi-backed services, the coverage doc must be a matrix. For vendor docs or flatter RPC APIs, a section inventory is acceptable until the service is implemented.

## Step 2: Create The SDK Module

Create the service module in `lab-apis`:

- add `pub mod <service>;` in `crates/lab-apis/src/lib.rs`
- add the feature flag in `crates/lab-apis/Cargo.toml`
- create `crates/lab-apis/src/<service>.rs`
- create `client.rs`, `types.rs`, and `error.rs`

In the client:

- construct the client from `Auth` and `HttpClient`
- keep all request building and response parsing here
- use typed request and response structs
- use service-specific ID newtypes where appropriate
- return `Result<T, <Service>Error>`

The canonical error and serialization contracts live in [ERRORS.md](./ERRORS.md) and [SERIALIZATION.md](./SERIALIZATION.md).

Do not add transport-specific concerns here. No `clap`, no MCP envelopes, no output formatting.

## Non-HTTP Capability Modules

Not every `lab` service is an HTTP API client.

Some modules are implemented primarily through our own code and may rely on:

- local filesystem access
- SSH-backed reads
- parsing local app data or config files
- shell or system-tool execution
- synthetic workflows that do not map to an upstream network API

For these modules:

- `lab-apis` still owns the core capability logic
- the shared `services` layer still owns operation schema, validation, client or target resolution, and surface-neutral execution
- CLI, MCP, and HTTP API remain thin adapters
- the module is not required to use `HttpClient`

What still applies:

- [DISPATCH.md](./DISPATCH.md)
- [OBSERVABILITY.md](./OBSERVABILITY.md)
- [ERRORS.md](./ERRORS.md)
- [SERIALIZATION.md](./SERIALIZATION.md)
- [TESTING.md](./TESTING.md)

What changes:

- health checks may use a lightweight local or SSH-backed capability probe instead of an HTTP request
- the source contract may be a documented file, parser, or workflow contract rather than an OpenAPI or vendor REST spec
- observability must instrument the real execution boundary for the module, not fabricate fake HTTP request logs

`extract` is the reference pattern for this class of module.

## Step 3: Add Observability

Observability is required before the service is considered online.

The canonical contract lives in [OBSERVABILITY.md](./OBSERVABILITY.md).

Minimum requirements:

- outbound service calls must flow through `HttpClient`
- the service must inherit dispatch context from CLI, MCP, and HTTP
- request logs must be traceable back to the invoking surface
- failure paths must preserve stable `kind` values and useful diagnostic detail
- logs must comply with the redaction rules

Do not treat observability as cleanup or post-implementation polish. Add it before public-surface verification.

## Step 4: Add Health

Every service must implement a health surface for `lab doctor` and `lab health`.

Use the lightest request that proves:

- the service is reachable
- auth is accepted
- version or status can be read if the API exposes it

The health check must be shorter-lived than ordinary requests and must not do destructive work.

Health checks must also be distinguishable in logs via the shared observability contract.

## Step 5: Add Feature Gating

Feature gating must be mirrored in both crates:

- `crates/lab-apis/Cargo.toml`
- `crates/lab/Cargo.toml`

`lab-apis` exports the service module behind the feature.
`lab` passes that feature through to `lab-apis`.

If the service must be available by default, add it to the `default` feature set in `crates/lab/Cargo.toml`.

## Step 6: Wire The CLI

Create `crates/lab/src/cli/<service>.rs` and keep it thin.

The CLI must:

- parse arguments with `clap`
- call the `lab-apis` client
- format output through `crate::output`
- respect destructive confirmation rules
- support `--json` through the common output layer

CLI output and machine-readable shape must follow [SERIALIZATION.md](./SERIALIZATION.md).

Then register it in `crates/lab/src/cli.rs`.

If the command is destructive, require `-y` / `--yes` or `--no-confirm`, and support `--dry-run` where the command semantics allow it.

CLI verification is not complete unless dispatch logs carry the required caller context from [OBSERVABILITY.md](./OBSERVABILITY.md).

## Step 7: Wire The MCP Dispatcher

Create `crates/lab/src/mcp/services/<service>.rs`.

The dispatcher must:

- define the `ActionSpec` catalog for the service
- parse `params.action`
- validate params
- call the client methods
- return `Result<Value, ToolError>`
- use the shared `ToolError` envelope on every failure

The canonical error and envelope behavior lives in [ERRORS.md](./ERRORS.md) and [SERIALIZATION.md](./SERIALIZATION.md).

Register the service in `crates/lab/src/mcp/registry.rs`.

Important rules:

- one MCP tool per service
- one `help` action and `schema` support are available through the shared dispatcher model
- destructive actions must be marked in `ActionSpec`
- the dispatcher must not contain business logic
- the dispatcher must not bypass elicitation for destructive actions

MCP verification is not complete unless dispatch logs carry the required caller context from [OBSERVABILITY.md](./OBSERVABILITY.md).

## Step 8: Wire The HTTP API

Create `crates/lab/src/api/services/<service>.rs`.

The HTTP route must mirror the MCP dispatch shape:

- one `POST /` handler
- `action` plus `params`
- same dispatch behavior as MCP
- same error envelope shape

HTTP error semantics must stay aligned with [ERRORS.md](./ERRORS.md) and [SERIALIZATION.md](./SERIALIZATION.md).

Then register the module in `crates/lab/src/api/services.rs`.

If the service has an HTTP router entry point elsewhere, keep it thin and route through the same service dispatch code.

HTTP verification is not complete unless request IDs and dispatch context are observable under the shared contract.

## Step 9: Register Metadata

Add the service metadata in the right places:

- `lab-apis` service `META`
- `crates/lab/src/tui/metadata.rs`
- any install or doctor logic that enumerates services

Metadata must include:

- canonical service name
- display name
- description
- category
- docs URL
- required env vars
- optional env vars
- default port

The TUI must not duplicate service-specific logic. It must render from metadata.

## Step 10: Add Config And Env Wiring

All config ownership stays in `lab`, not `lab-apis`.

Add the service env variables in the config layer:

- `SERVICE_URL`
- `SERVICE_API_KEY` or `SERVICE_TOKEN`
- optional `SERVICE_USERNAME` / `SERVICE_PASSWORD` if the service uses basic auth

If the service supports multiple instances, follow the established pattern:

- `SERVICE_URL` for the default instance
- `SERVICE_<LABEL>_URL` for additional instances

Do not hardcode instance names in the service layer.

## Step 11: Update Docs

Update these docs as part of the same change:

- `docs/coverage/<service>.md`
- `docs/SERVICES.md` if the service inventory or workflow changes
- `docs/README.md` if a new docs entry is needed
- `docs/CONFIG.md` if the service introduces new env or instance naming rules
- `docs/MCP.md` or `docs/CLI.md` if the public surface model changes
- `docs/OBSERVABILITY.md` if the shared contract changes
- `docs/ERRORS.md` if the shared error contract changes
- `docs/SERIALIZATION.md` if the shared serde or envelope contract changes

## Step 12: Test And Verify

Verification must cover behavior, observability, error shape, and serialization shape.

Required minimums:

- unit tests for the SDK and shared helpers
- non-destructive CLI verification against a live instance when possible
- matching non-destructive MCP verification when possible
- evidence that one successful path and one failing path are traceable end to end
- evidence that error kinds and envelopes match the shared contract where touched
- evidence that machine-readable output matches the intended shape where touched

If a service works but its request path is still a black box, it is not fully online.

The coverage doc must stay aligned with the actual implementation counts and file paths.

## Completion Checklist

A service is ready when:

- the SDK compiles and tests pass
- the CLI command exists and runs
- the MCP tool exists and dispatches correctly
- the HTTP route exists and matches MCP behavior
- feature flags are wired in both crates
- metadata and discovery are wired
- the coverage doc is up to date
- error and serialization behavior match the shared contracts
- live verification is done or explicitly marked as pending

## Common Failure Modes

- putting business logic in CLI or MCP layers
- forgetting to add the feature passthrough in `crates/lab/Cargo.toml`
- registering the service in one surface but not the others
- leaving the coverage doc path or counts stale
- treating compiled-in support as the same thing as live verification
- adding new config behavior in `lab-apis` instead of `lab`

## Related Docs

- [ARCH.md](./ARCH.md)
- [SERVICES.md](./SERVICES.md)
- [CONFIG.md](./CONFIG.md)
- [CLI.md](./CLI.md)
- [MCP.md](./MCP.md)
- [OPERATIONS.md](./OPERATIONS.md)
- [CONVENTIONS.md](./CONVENTIONS.md)
- [OBSERVABILITY.md](./OBSERVABILITY.md)
- [ERRORS.md](./ERRORS.md)
- [SERIALIZATION.md](./SERIALIZATION.md)
