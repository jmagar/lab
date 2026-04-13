# Service Onboarding

This is the canonical end-to-end workflow for bringing a new service online in `lab`.

Use this when adding a brand-new integration or when completing a partially implemented service.

## Goal

Bringing a service online means all of the following are true:

- the `lab-apis` client exists and owns the service logic
- the service is feature-gated in both crates
- the CLI shim exists
- the MCP dispatcher exists
- the API dispatcher exists
- the service is registered in the MCP registry, CLI router, API router, and TUI metadata
- the service has a coverage doc under `docs/coverage/`
- the request path is observable end to end under the shared observability contract
- the implementation is tested at the unit level and verified against a live instance when possible

The service is not considered done if only one surface works.

For new service work, prefer the onboarding scaffold first and then run the
onboarding audit before claiming the service is aligned with repo conventions.
`lab scaffold service` should generate the initial shape; `lab audit onboarding`
should verify it; `cargo test --all-features` is the final gate.

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
- CLI, MCP, and API are thin adapters over `dispatch/` — **they never call `lab-apis` directly**
- `dispatch/<service>/` is the shared semantic layer; all three surfaces route through it
- `lab-apis` must not read config files or ambient env directly
- env reads, instance resolution, and client construction belong in `dispatch/<service>/client.rs`
- every service is feature-gated
- destructive operations must be marked at the action level and honored consistently across surfaces
- one MCP tool per service, with `action`-based dispatch
- use `dispatch::helpers::env_non_empty` for all env reads in dispatch — do not inline `std::env::var(...).ok().filter(...)`

The canonical adapter and dependency-direction rules live in [DISPATCH.md](./DISPATCH.md).

If you are writing logic in `crates/lab/src/cli/<service>.rs`, `crates/lab/src/mcp/services/<service>.rs`, or `crates/lab/src/api/services/<service>.rs`, the logic probably belongs in `crates/lab/src/dispatch/<service>/` or `lab-apis` instead.

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

- `crates/lab/src/dispatch/<service>.rs` — entry point: submodule declarations, re-exports, and unit tests
- `crates/lab/src/dispatch/<service>/catalog.rs` — single source of truth for all `ActionSpec` / `ParamSpec` definitions
- `crates/lab/src/dispatch/<service>/client.rs` — env lookup, instance resolution, auth construction, and `not_configured_error()`
- `crates/lab/src/dispatch/<service>/params.rs` — all coercion from `serde_json::Value` to typed SDK request structs
- `crates/lab/src/dispatch/<service>/dispatch.rs` — top-level action routing; exposes `dispatch()` and `dispatch_with_client()`
- `crates/lab/src/cli/<service>.rs`
- `crates/lab/src/mcp/services/<service>.rs`
- `crates/lab/src/api/services/<service>.rs`

Registry and metadata wiring live in:

- `crates/lab/src/cli.rs` — `pub mod <service>;` declaration and subcommand registration
- `crates/lab/src/mcp/registry.rs` — runtime registration via `build_default_registry()`
- `crates/lab/src/mcp/services.rs` — `pub mod <service>;` module declaration (distinct from `registry.rs`)
- `crates/lab/src/api/services.rs` — `pub mod <service>;` module declaration
- `crates/lab/src/api/router.rs` — feature-gated `.nest("/v1/<service>", services::<service>::routes(state.clone()))` block
- `crates/lab/src/tui/metadata.rs`

The catalog (`lab.help` meta-tool, `lab://catalog` resource, `lab help` CLI) is built automatically from `ToolRegistry` via `build_catalog()` — no separate catalog registration step is needed beyond `mcp/registry.rs`.

Feature gating and re-exports live in:

- `crates/lab-apis/src/lib.rs`
- `crates/lab/Cargo.toml`

## Dispatch Layer Deep Dive

The `dispatch/<service>/` directory is the shared semantic core. Every surface (CLI, MCP, API) calls into it — they never reach `lab-apis` directly. This section documents the patterns that every dispatch module must follow.

### The Two-Function Pattern

Every `dispatch.rs` must expose two public functions:

```rust
/// Called by MCP (and optionally typed CLI). Builds the client from env, then delegates.
pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help"   => Ok(help_payload("<service>", ACTIONS)),
        "schema" => { let a = require_str(&params, "action")?; action_schema(ACTIONS, a) }
        _        => dispatch_with_client(&require_client()?, action, params).await,
    }
}

/// Called by the API handler with a pre-built client from AppState.
/// Also called by MCP/CLI via `dispatch()` above.
pub async fn dispatch_with_client(
    client: &<Service>Client,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        "resource.verb" => { /* ... */ }
        unknown => Err(ToolError::UnknownAction {
            service: "<service>".into(),
            action: unknown.to_string(),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
        }),
    }
}
```

Why two functions: the API handler holds a pre-built `Option<ServiceClient>` from `AppState` (resolved once at startup). It calls `dispatch_with_client` directly to avoid re-reading env vars per request. MCP and CLI go through `dispatch`, which builds the client on demand.

### The Entry-Point (`dispatch/<service>.rs`)

The entry-point file has exactly three responsibilities:

1. Declare the four submodules
2. Re-export the public surface with `#[allow(unused_imports)]`
3. Own the unit tests

```rust
//! Shared dispatch layer for the `<service>` service.

mod catalog;
mod client;
mod dispatch;
mod params;

pub use catalog::ACTIONS;
#[allow(unused_imports)]
pub use client::client_from_env;
#[allow(unused_imports)]
pub use dispatch::{dispatch, dispatch_with_client};

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn catalog_includes_first_wave_actions() {
        let names: Vec<&str> = ACTIONS.iter().map(|a| a.name).collect();
        assert!(names.contains(&"resource.verb"));
    }

    #[tokio::test]
    async fn help_lists_primary_action() {
        let value = dispatch("help", serde_json::json!({})).await.unwrap();
        let actions = value["actions"].as_array().unwrap();
        assert!(actions.iter().any(|a| a["name"] == "resource.verb"));
    }

    #[tokio::test]
    async fn dispatch_with_client_round_trips() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/expected/path"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;
        let client = lab_apis::<service>::<Service>Client::new(&server.uri(), Auth::None).unwrap();
        let value = dispatch_with_client(&client, "resource.verb", serde_json::json!({}))
            .await
            .unwrap();
        assert!(value.is_object());
    }
}
```

The `#[allow(unused_imports)]` on the re-exports is required. Not every compilation path uses every re-export (e.g. the API handler uses `dispatch_with_client` but not `dispatch`), and clippy will fail without it.

Three standard unit tests belong in every entry-point:

1. `catalog_includes_<key>_actions` — verifies the catalog compiles and contains at least the expected actions
2. `help_lists_<primary_action>` — smoke-tests the `help` action end to end
3. `dispatch_with_client_<describes_behavior>` — one `wiremock` round-trip proving the dispatch→client→HTTP chain works

A fourth test covering a **failing path** is also required. The most portable option is an unknown-action error, which requires no mock server:

```rust
#[tokio::test]
async fn dispatch_returns_unknown_action_error() {
    let err = dispatch("not.a.real.action", serde_json::json!({}))
        .await
        .unwrap_err();
    assert!(matches!(err, ToolError::UnknownAction { .. }));
}
```

Other acceptable failing paths: missing required param (`ToolError::MissingParam`), invalid param type (`ToolError::InvalidParam`), or a wiremock that returns 401 to verify auth failure handling. At least one must be present.

### `client.rs` — Env, Auth, and `not_configured_error()`

Every `client.rs` must expose three things:

```rust
use crate::dispatch::helpers::env_non_empty;

/// Build a client from env. Returns `None` if any required var is absent or empty.
/// Called by `AppState` at startup — keep pure (no logging, no side effects).
pub fn client_from_env() -> Option<<Service>Client> {
    let url = env_non_empty("<SERVICE>_URL")?;
    let key = env_non_empty("<SERVICE>_API_KEY")?;
    <Service>Client::new(&url, Auth::ApiKey { header: "X-Api-Key".into(), key }).ok()
}

/// Return a client or a structured `internal_error` if not configured.
/// Used by MCP and CLI when AppState is not available.
pub fn require_client() -> Result<<Service>Client, ToolError> {
    client_from_env().ok_or_else(not_configured_error)
}

/// Structured error for callers that hold a pre-built `Option<ServiceClient>`.
/// The API handler calls this directly instead of re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "<SERVICE>_URL or <SERVICE>_API_KEY not configured".into(),
    }
}
```

The API handler holds `Option<ServiceClient>` from `AppState`. It cannot call `require_client()` (that reads env). Exposing `not_configured_error()` separately lets the API handler produce the same structured error without re-reading env or duplicating the message string.

Always use `env_non_empty` from `dispatch::helpers` — not `std::env::var(...).ok().filter(|v| !v.is_empty())` inline.

**Wiring `From<ServiceError>` for `ToolError`:**

All `From<ServiceError> for ToolError` impls must live in `crates/lab/src/dispatch/error.rs`, feature-gated. Use the `impl_tool_error_from!` macro defined there — do not write the impl by hand:

```rust
// In crates/lab/src/dispatch/error.rs:
#[cfg(feature = "foo")]
impl_tool_error_from!(lab_apis::foo::FooError);
```

Do not add `From` impls in `mcp/services/`, `api/services/`, or `cli/` — that creates multiple conversion paths and drift.

**`ToolError` serialization invariants:**

- Never add `#[derive(Serialize)]` to `ToolError` — it serializes via its `Display` impl, which emits JSON
- `ToolError::Display` always produces JSON, not human text — surfaces that want human-readable errors must format them explicitly
- `IntoResponse` for `ToolError` is shared between MCP and HTTP — do not write per-surface response conversions

**Choosing the right `Auth` variant:**

| Service auth style | `Auth` variant |
|--------------------|----------------|
| API key in a request header (e.g. `X-Api-Key`) | `Auth::ApiKey { header: "X-Api-Key".into(), key }` |
| Bearer token in `Authorization` header | `Auth::Bearer { token }` |
| HTTP basic auth (username + password) | `Auth::Basic { username, password }` |
| No auth required (public endpoints, health probes) | `Auth::None` |

Check the upstream API docs or the `lab-apis` client constructor to confirm which variant the service expects. Using `Auth::Bearer` for a service that expects `Auth::ApiKey` (or vice versa) will produce silent 401s that are easy to miss in tests.

### Multi-Role Auth

Some services use different credentials for different operation classes (e.g. one token for sending, another for management). When this applies, model the multiple roles explicitly rather than creating multiple clients at call sites:

```rust
#[derive(Clone)]
pub struct <Service>Clients {
    health: Arc<<Service>Client>,    // no auth — health probe only
    write:  Option<Arc<<Service>Client>>,  // write token
    read:   Option<Arc<<Service>Client>>,  // read/management token
}
```

Expose accessor methods (`health()`, `write()`, `read()`) that return references. Keep the construction in `clients_from_env()`. Expose a `not_configured_error()` so the API handler can produce the same error without re-reading env.

See `crates/lab/src/dispatch/gotify/client.rs` for the reference pattern.

### `catalog.rs` — `help` and `schema` Must Be Explicit

The `ACTIONS` array is the single source of truth for what `help` and `schema` return. The shared dispatcher routes `help` and `schema` automatically, but they must still appear in `ACTIONS` so the `help` response lists them. Every catalog must start with:

```rust
pub const ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        name: "help",
        description: "Show this action catalog",
        destructive: false,
        returns: "Catalog",
        params: &[],
    },
    ActionSpec {
        name: "schema",
        description: "Return the parameter schema for a named action",
        destructive: false,
        returns: "Schema",
        params: &[ParamSpec {
            name: "action",
            ty: "string",
            required: true,
            description: "Action name to describe",
        }],
    },
    // ... service-specific actions follow
];
```

Omitting them from `ACTIONS` means `help` will return an incomplete catalog — a service that lists `resource.verb` but not `help` itself is confusing to agents.

**Action name stability:** Action names are part of the machine-facing public contract. Renaming an action is a spec change — it requires simultaneous updates to `catalog.rs`, `docs/coverage/<service>.md`, and any docs that reference the action. Do not rename actions casually.

### `params.rs` — All Coercion Lives Here

`params.rs` owns every conversion from `serde_json::Value` to a typed SDK request struct. No `dispatch.rs` arm should inline param coercion. If building a request requires more than reading one string, it belongs in a `params.rs` helper.

```rust
// params.rs
pub fn create_request_from_params(params: &Value) -> Result<CreateRequest, ToolError> {
    let name = require_str(params, "name")?.to_string();
    let priority = optional_u32(params, "priority")?;
    Ok(CreateRequest { name, priority })
}

// dispatch.rs arm — stays thin
"resource.create" => {
    let req = create_request_from_params(&params)?;
    to_json(client.create(req).await?)
}
```

Use the helpers from `dispatch::helpers`: `require_str`, `optional_str`, `require_i64`, `optional_u32`, `optional_u32_max`, `body_from_params`, `object_without`. Do not write custom extraction logic when a helper already exists.

## Migrating From A Stub

Some services exist as stub dispatch files (~28 lines) — a flat `.rs` file that always returns `not_implemented`. A stub looks like this:

```rust
// dispatch/overseerr.rs — STUB, not yet implemented
use serde_json::Value;
use crate::dispatch::error::ToolError;

pub async fn dispatch(_action: &str, _params: Value) -> Result<Value, ToolError> {
    Err(ToolError::Sdk {
        sdk_kind: "not_implemented".into(),
        message: "overseerr dispatch not yet implemented".into(),
    })
}
```

When migrating a stub to a full implementation:

1. Delete the stub `.rs` file
2. Create the `dispatch/<service>/` directory with `catalog.rs`, `client.rs`, `params.rs`, `dispatch.rs`
3. Create the new `dispatch/<service>.rs` entry-point with submodule declarations, re-exports, and unit tests
4. Do **not** attempt to split from a single large file after the fact — build directory-first from the start

The stub registration in `mcp/registry.rs` and `api/services.rs` may already exist. Check before adding duplicate entries.

## Recommended Order

Bring a service online in this order:

1. verify or add the upstream API spec
2. create the `lab-apis` module
3. define types and the service error
4. implement the client methods
5. add observability
6. implement health
7. add feature flags and module re-exports
8. create the dispatch module (`catalog.rs`, `client.rs`, `params.rs`, `dispatch.rs`, entry-point with unit tests)
9. wire env vars and auth in `dispatch/<service>/client.rs`
10. wire the CLI shim
11. wire the MCP dispatcher
12. wire the API dispatcher
13. register metadata and runtime discovery
14. add and update docs
15. run tests and live verification

That order keeps the service boundary stable before the public surfaces are wired. The dispatch module (step 8) and env wiring (step 9) must exist before any surface can call into them — do not start steps 10–12 until both are done.

**Note on step numbering:** The detailed sections below are labeled "Step 1" through "Step 12" and consolidate some of the above into logical groupings (e.g. "Step 2: Create The SDK Module" covers items 2–4 above). Use the Recommended Order list for sequencing and the Step sections for implementation detail.

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
- what the API handler exposes
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

All public items in `lab-apis` must have rustdoc comments. Client methods should include a brief description and, where practical, a compilable example. `cargo doc --no-deps --all-features` must pass warning-free — this is enforced in CI.

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
- call `dispatch::<service>::dispatch()` — not `lab-apis` directly
- format output through `crate::output`
- respect destructive confirmation rules
- support `--json` through the common output layer

CLI output and machine-readable shape must follow [SERIALIZATION.md](./SERIALIZATION.md).

Then register it in `crates/lab/src/cli.rs`.

If the command is destructive, require `-y` / `--yes` to bypass confirmation. `--no-confirm` is defined in the CLI contract as an alias for `-y` but has not yet been implemented across all services — wire it only when the shared implementation is available. Support `--dry-run` where the command semantics allow it (prints what would happen without calling the client).

CLI verification is not complete unless dispatch logs carry the required caller context from [OBSERVABILITY.md](./OBSERVABILITY.md).

## Step 7: Wire The MCP Dispatcher

Create `crates/lab/src/mcp/services/<service>.rs`.

The dispatcher must:

- call `dispatch::<service>::dispatch(action, params)` and return its result
- use the shared `ToolError` envelope on every failure

The MCP dispatcher is a thin shim. All action routing, param validation, client construction, and business logic live in `dispatch/<service>/`. The MCP file itself should contain almost no logic.

The canonical error and envelope behavior lives in [ERRORS.md](./ERRORS.md) and [SERIALIZATION.md](./SERIALIZATION.md).

Register the service in `crates/lab/src/mcp/registry.rs`.

Important rules:

- one MCP tool per service
- `help` and `schema` are routed by `dispatch()` — both must be declared in `catalog.rs`'s `ACTIONS` array so the `help` response lists them
- destructive actions must be marked `destructive: true` in `ActionSpec`
- the MCP file must not contain business logic, param coercion, or its own action catalog — it projects from `dispatch/<service>/catalog.rs` only
- MCP elicitation for destructive ops is **not yet implemented** — the `ActionSpec.destructive` flag is honored by the HTTP surface only (via `handle_action` in `api/services/helpers.rs`); MCP currently dispatches destructive actions without a confirmation gate

Additionally, add `pub mod <service>;` to `crates/lab/src/mcp/services.rs` — this module declaration is required alongside the `mcp/registry.rs` registration.

MCP verification is not complete unless dispatch logs carry the required caller context from [OBSERVABILITY.md](./OBSERVABILITY.md).

## Step 8: Wire The API

Create `crates/lab/src/api/services/<service>.rs`.

Wiring the API requires three distinct steps:

**1. Create the handler (`api/services/<service>.rs`)**

Use `handle_action` from `api/services/helpers.rs` — this is the canonical wrapper that handles `confirmation_required` for destructive actions, request logging, and error translation. Do not write a raw `dispatch_with_client` call without it:

```rust
use crate::api::services::helpers::handle_action;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", post(handler))
        .with_state(state)
}

async fn handler(State(state): State<AppState>, Json(req): Json<ActionRequest>) -> impl IntoResponse {
    let client = state.clients.<service>.clone();
    handle_action(
        "<service>",
        "api",
        None,
        req,
        dispatch::<service>::ACTIONS,
        move |action, params| async move {
            let Some(client) = client.as_ref() else {
                return Err(crate::dispatch::<service>::not_configured_error());
            };
            crate::dispatch::<service>::dispatch_with_client(client, &action, params).await
        },
    ).await
}
```

The `handle_action` wrapper gates destructive actions behind `params["confirm"] == true`. Missing this gate returns `400` with `kind: "confirmation_required"` instead of executing the destructive operation — which is the correct HTTP behavior (HTTP equivalent of the MCP elicitation flow).

**2. Add the module declaration (`api/services.rs`)**

Add `pub mod <service>;` (feature-gated).

**3. Add the nest call (`api/router.rs`)**

Add a feature-gated block to mount the routes:

```rust
#[cfg(feature = "<service>")]
{
    v1 = v1.nest("/v1/<service>", services::<service>::routes(state.clone()));
}
```

Without this block the module compiles but the routes are never mounted.

**4. Wire the `AppState` client field (`api/state.rs`)**

Add `pub <service>: Option<Arc<<Service>Client>>` to `AppState`'s clients struct and call `dispatch::<service>::client_from_env()` in the `from_env()` constructor. Without this, `state.clients.<service>` will not exist and the handler cannot get a client.

API handlers run inside the shared middleware stack from `router.rs` (request ID, tracing, 30s timeout, gzip, CORS). Do not add your own timeout, CORS, or compression logic — the middleware handles it automatically.

API error semantics must stay aligned with [ERRORS.md](./ERRORS.md) and [SERIALIZATION.md](./SERIALIZATION.md).

API verification is not complete unless request IDs and dispatch context are observable under the shared contract.

## Step 9: Register Metadata

Four locations must all be touched — missing any one causes silent disappearance from a surface:

1. **`crates/lab/src/mcp/registry.rs`** — runtime registration in `build_default_registry()`
2. **`crates/lab/src/mcp/services.rs`** — `pub mod <service>;` module declaration
3. **`crates/lab/src/api/services.rs`** — `pub mod <service>;` module declaration
4. **`crates/lab/src/api/router.rs`** — feature-gated `.nest()` block (see Step 8)

Additionally:

- **`lab-apis` service `META`** — `PluginMeta` constant in the service entry-point file
- **`crates/lab/src/tui/metadata.rs`** — TUI plugin manager entry
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

## Step 10: Finalize Config And Env Wiring

The env var names used in `dispatch/<service>/client.rs` (step 9 of Recommended Order) are the primary config artifact. This step is about auditing completeness and adding any secondary wiring.

Verify the following are consistent across the codebase:

- env var names in `dispatch/<service>/client.rs` match what `docs/CONFIG.md` documents
- the service `META` in `lab-apis` declares the same vars in `required_env` / `optional_env`
- `~/.lab/.env` examples or docs reference the correct names

Standard naming convention:

- `SERVICE_URL`
- `SERVICE_API_KEY` or `SERVICE_TOKEN`
- optional `SERVICE_USERNAME` / `SERVICE_PASSWORD` for basic auth

If the service supports multiple instances, follow the established pattern:

- `SERVICE_URL` for the default instance
- `SERVICE_<LABEL>_URL` for additional instances

Do not hardcode instance names in the dispatch layer. All config ownership stays in `lab`, not `lab-apis`.

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

**TDD rule:** Write the failing test first, then the implementation. This applies to new service endpoints, contract changes, bug fixes, and refactors. If true TDD sequencing is not possible, document the reason explicitly in the task or review note.

**A service is not done until all three surfaces have been smoke-tested live.** For HTTP-backed services, all three surfaces (CLI, MCP, API) must be smoke-tested live. For non-HTTP modules (such as `extract`), adapt the verification to the actual execution boundary. **Destructive actions do not require live testing** — cover them with mocked unit tests and wiremock fixtures only; do not run destructive live tests by default.

**Test runner:** Use `cargo nextest run` for all test execution. The standard full-crate verification command is:

```bash
cargo nextest run --manifest-path crates/lab/Cargo.toml --all-features
```

### Required: SDK Tests (in `lab-apis`)

Unit tests for the SDK layer live in `crates/lab-apis/tests/<service>_client.rs` (integration test files) or in `#[cfg(test)]` modules within the source files. These tests must use `wiremock` and must not depend on real running services. Required coverage:

- request construction round-trip (params → HTTP request)
- response decoding (HTTP response → typed struct)
- transport error mapping (non-200 → `ServiceError`)

### Required: Dispatch Unit Tests (in `dispatch/<service>.rs`)

Unit tests live in `crates/lab/src/dispatch/<service>.rs` (the entry-point). Every service must have at minimum these four tests:

1. **Catalog test** — verifies `ACTIONS` compiles and contains the expected action names
2. **Help test** — calls `dispatch("help", json!({}))` and asserts the action list contains at least one service-specific action
3. **Round-trip test** — starts a `MockServer`, builds a client pointed at it, calls `dispatch_with_client`, and asserts the response shape
4. **Failing path test** — at minimum, `dispatch("not.a.real.action", json!({}))` returns `ToolError::UnknownAction`; see the Deep Dive section for other acceptable failing paths

Additional required evidence:
- error kinds and envelopes match the shared contract where touched
- machine-readable output matches the intended shape where touched

### Required: MCP Adapter Tests (in `mcp/services/<service>.rs`)

Minimum coverage:
- MCP success envelope shape: `ok: true`, `service`, `action`, `data` fields present
- MCP error envelope shape: `ok: false`, `kind`, `message` fields present for `unknown_action`

### Required: API Adapter Tests (in `api/services/<service>.rs`)

Minimum coverage:
- HTTP 200 returned for a valid action with real data in the response body
- HTTP 400 returned for an unknown action with `kind: "unknown_action"` in the JSON body
- HTTP 400 returned for a destructive action without `confirm: true` with `kind: "confirmation_required"` (if applicable)

### Required: Live CLI Smoke Tests

Run non-destructive CLI commands directly against the running service instance. Example:

```bash
lab linkding bookmarks.list
lab linkding tags.list
lab linkding user.profile
```

Verify that:
- the command executes without error
- real data is returned from the live service
- output is human-readable by default and machine-readable with `--json`

### Required: Live MCP Smoke Tests (via mcporter)

`mcporter` is a standalone CLI tool for calling MCP tools against a running `lab` MCP server. It is not part of the `lab` binary. Start the lab MCP server first (`lab serve --transport stdio` or `--transport sse`), then use `mcporter call` to invoke tools:


```bash
mcporter call lab.linkding action=bookmarks.list
mcporter call lab.paperless action=statistics
mcporter call lab.prowlarr action=system.status
```

Verify that:
- `"ok": true` is returned
- `"data"` contains real service data
- the envelope shape matches the shared serialization contract

### Required: Live API Smoke Tests (via curl)

Start the lab API server (`lab serve --transport http`) and hit each service endpoint:

```bash
TOKEN=<LAB_MCP_HTTP_TOKEN from ~/.lab/.env>
curl -s -X POST http://127.0.0.1:8400/v1/<service> \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"action":"<action>","params":{}}'
```

Verify that:
- HTTP 200 is returned for valid actions
- the response body contains real data from the live service
- HTTP 400 is returned for unknown actions (structured error envelope)

### Required: Coverage Doc Update

After live testing, update `docs/coverage/<service>.md`:

- add a **Live Test Evidence** section with a table of surface / command / result
- change `⬜` symbols to `✅` for actions that have been successfully live-tested
- record the date and service version tested against

If a service works but its request path is still a black box, it is not fully online.

The coverage doc must stay aligned with the actual implementation counts and file paths.

## Completion Checklist

A service is ready when:

- the SDK compiles and tests pass
- SDK unit tests exist in `lab-apis` covering request, response, and error paths
- dispatch unit tests exist (4 standard tests including one failing path)
- MCP and API adapter tests exist
- the CLI command exists and runs
- the MCP tool exists and dispatches correctly
- the API route exists and matches MCP behavior (all four wiring locations complete)
- feature flags are wired in both crates
- metadata and discovery are wired (all four registration locations)
- observability verified: dispatch events emitted, secrets not logged, `operation = "health"` present on health probes
- the coverage doc is up to date with Live Test Evidence
- error and serialization behavior match the shared contracts
- live verification is done for non-destructive paths, or explicitly marked as pending

## Common Failure Modes

- putting business logic in CLI or MCP layers instead of `dispatch/<service>/`
- calling `lab-apis` directly from CLI, MCP, or API instead of going through `dispatch/`
- CLI or MCP calling each other (`api -> mcp`, `cli -> mcp`, etc. are all forbidden dependency directions)
- forgetting `#[allow(unused_imports)]` on dispatch entry-point re-exports (causes clippy failure in some compilation configurations)
- omitting `help` and `schema` from `ACTIONS` in `catalog.rs` (they still need to be declared)
- inlining param coercion in `dispatch.rs` instead of delegating to `params.rs`
- copying the `ACTIONS` array instead of re-exporting from `catalog.rs` — two copies drift, and elicitation silently skips newly-added destructive actions on the stale copy
- using `std::env::var(...).ok().filter(...)` inline instead of `env_non_empty`
- the API handler calling `require_client()` (re-reads env per request) instead of using the `AppState` client and `not_configured_error()`
- forgetting `not_configured_error()` — duplicating the error message string at call sites
- writing `From<ServiceError> for ToolError` in surface code instead of `dispatch/error.rs` using `impl_tool_error_from!`
- creating a `DispatchError` type as a parallel to `ToolError` — this was considered and explicitly rejected; use `ToolError` directly
- logging `params` at any dispatch boundary — some actions pass plaintext credentials through `params`
- calling `handle_action` without checking that the client is `Some` — unwrapping `None` panics in the API handler
- forgetting to add the `router.rs` `.nest()` block — routes compile but are never mounted
- adding `pub mod <service>` to `api/services.rs` but not to `mcp/services.rs`, or vice versa
- forgetting to add the feature passthrough in `crates/lab/Cargo.toml`
- leaving the coverage doc path or counts stale
- treating compiled-in support as the same thing as live verification
- adding new config behavior in `lab-apis` instead of `lab`
- building a stub migration as one large file instead of directory-first from day one
- adding `#[derive(Serialize)]` to `ToolError` — it serializes via `Display` (JSON), not serde

## Non-HTTP Capability Modules

Not every `lab` service is an API client. This section applies only to modules that don't map to an upstream network API — skip it for standard HTTP-backed services.

Some modules are implemented primarily through our own code and may rely on:

- local filesystem access
- SSH-backed reads
- parsing local app data or config files
- shell or system-tool execution
- synthetic workflows that do not map to an upstream network API

For these modules:

- `lab-apis` still owns the core capability logic
- the shared `dispatch` layer still owns operation schema, validation, client or target resolution, and surface-neutral execution
- CLI, MCP, and API remain thin adapters
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

## Related Docs

- [ARCH.md](./ARCH.md)
- [DISPATCH.md](./DISPATCH.md)
- [SERVICES.md](./SERVICES.md)
- [CONFIG.md](./CONFIG.md)
- [CLI.md](./CLI.md)
- [MCP.md](./MCP.md)
- [RMCP.md](./RMCP.md)
- [OPERATIONS.md](./OPERATIONS.md)
- [CONVENTIONS.md](./CONVENTIONS.md)
- [OBSERVABILITY.md](./OBSERVABILITY.md)
- [ERRORS.md](./ERRORS.md)
- [SERIALIZATION.md](./SERIALIZATION.md)
- [TESTING.md](./TESTING.md)
- [EXTRACT.md](./EXTRACT.md)
