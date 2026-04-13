# dispatch/ — shared dispatch layer

This directory is the shared semantic layer between the product adapters and `lab-apis`.

## Core Rule

`cli/`, `mcp/services/`, and `api/services/` are thin adapters over `dispatch/`.

If multiple surfaces need the same action semantics, that code belongs here.

## Required Service Layout

Every migrated service must be directory-first from day one.

Required shape:

- `crates/lab/src/dispatch/<service>.rs`
- `crates/lab/src/dispatch/<service>/catalog.rs`
- `crates/lab/src/dispatch/<service>/client.rs`
- `crates/lab/src/dispatch/<service>/params.rs`
- `crates/lab/src/dispatch/<service>/dispatch.rs`

Optional:

- extra domain modules such as `devices.rs`, `wifi.rs`, `movies.rs`, `queue.rs`

Rules:

- `<service>.rs` is a thin entrypoint only
- `catalog.rs` is the single source of truth for `ActionSpec` / `ParamSpec`
- `client.rs` owns env lookup, instance lookup, auth construction, and client construction
- `params.rs` owns param coercion and request-body/query construction helpers
- `dispatch.rs` owns top-level action routing and help payload generation
- broad services may add domain modules, but not instead of the standard four files

Do not start a migrated service as one large file and split it later.

## Ownership

`dispatch/` owns:

- action catalogs
- param metadata and validation
- destructive metadata
- client and instance resolution
- shared action execution
- surface-neutral `Result<Value, ToolError>` behavior

`dispatch/` does not own:

- `clap` parsing
- MCP registration or envelopes
- API routing or status mapping
- output formatting
- upstream request/response parsing that belongs in `lab-apis`

## Error Rule

`ToolError` lives in `dispatch/error.rs`.

Keep all `From<ServiceError> for ToolError` impls there.
Do not create adapter-local conversion paths.

## Canonical `client.rs` Template

Every `dispatch/<service>/client.rs` must follow this exact shape:

```rust
use lab_apis::<service>::<Service>Client;
use lab_apis::core::Auth;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Build a `<Service>` client from the default-instance env vars.
///
/// Returns `None` if any required env var is absent or empty.
/// Called by `AppState` at startup — keep pure (no side effects, no logging).
pub fn client_from_env() -> Option<<Service>Client> {
    let url = env_non_empty("<SERVICE>_URL")?;
    let key = env_non_empty("<SERVICE>_API_KEY")?;
    <Service>Client::new(&url, Auth::ApiKey { header: "X-Api-Key".into(), key }).ok()
}

/// Return a client or a structured `internal_error` if not configured.
/// Used by MCP and CLI when `AppState` is not available.
pub fn require_client() -> Result<<Service>Client, ToolError> {
    client_from_env().ok_or_else(not_configured_error)
}

/// Structured error for callers that hold a pre-built `Option<ServiceClient>`.
/// The API handler calls this directly instead of re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "<SERVICE>_URL or <SERVICE>_API_KEY not configured".to_string(),
    }
}
```

Rules:
- `client_from_env()` is called by `AppState::ServiceClients::from_env()` at startup — keep it pure (no side effects, no logging).
- `require_client()` is the MCP/CLI fallback when `AppState` is not available.
- `not_configured_error()` is exposed separately so API handlers can produce the same structured error without re-reading env vars.
- Always use `env_non_empty` — never inline `std::env::var(...).ok().filter(|v| !v.is_empty())`.
- Never read env vars inside `dispatch.rs` or `params.rs` — always go through `client.rs`.
- When the service supports multiple instances, use `InstancePool<C>` from `dispatch::helpers` instead of a bespoke `OnceLock`. Implement `client_from_instance(label: Option<&str>)` as the public entry point:

```rust
use std::sync::OnceLock;
use crate::dispatch::helpers::{env_non_empty, InstancePool};

static POOL: OnceLock<InstancePool<<Service>Client>> = OnceLock::new();

fn pool() -> &'static InstancePool<<Service>Client> {
    POOL.get_or_init(|| {
        InstancePool::build("<SERVICE>", |url, key| {
            <Service>Client::new(&url, Auth::ApiKey { header: "X-Api-Key".into(), key }).ok()
        })
    })
}

pub fn client_from_instance(label: Option<&str>) -> Result<&'static <Service>Client, ToolError> {
    pool().resolve(label)
}
```

`InstancePool::build(prefix, closure)` scans for `{PREFIX}_URL` (default instance) and `{PREFIX}_{LABEL}_URL` (named instances) at first call, caching all clients in a single `OnceLock`. `resolve(None)` returns the default instance; `resolve(Some("label"))` returns the named one. Both return `ToolError::UnknownInstance` if the label is absent.

> **Header casing:** The default is `X-Api-Key` (Servarr convention). Some APIs enforce specific casing:
> - Unraid: `X-API-Key` — matches the Unraid server's exact validation
> - UniFi: `X-API-KEY` — all caps, matches UniFi Network Application spec
> HTTP headers are case-insensitive on the wire, but some servers validate exact casing.
> Check the upstream API spec before setting.

## `dispatch.rs` Template

Every `dispatch/<service>/dispatch.rs` must handle the two built-in actions before
dispatching to service-specific logic:

```rust
use crate::dispatch::helpers::{action_schema, help_payload, require_str};
use super::catalog::ACTIONS;
use super::client::require_client;

pub async fn dispatch(action: &str, params: Value) -> Result<Value, ToolError> {
    match action {
        "help"   => Ok(help_payload("<service>", ACTIONS)),
        "schema" => { let a = require_str(&params, "action")?; action_schema(ACTIONS, a) }
        _        => dispatch_with_client(&require_client()?, action, params).await,
    }
}

pub async fn dispatch_with_client(
    client: &<Service>Client,
    action: &str,
    params: Value,
) -> Result<Value, ToolError> {
    match action {
        // ... service-specific arms ...
        unknown => Err(ToolError::UnknownAction {
            service: "<service>".into(),
            action: unknown.to_string(),
            valid: ACTIONS.iter().map(|a| a.name.to_string()).collect(),
        }),
    }
}
```

`dispatch_with_client` is what the API handler calls with the pre-built client from
`AppState`. `dispatch` is what MCP and CLI call.

## Naming

Use `API` for the product surface name in comments and docs.

Reserve `HTTP` for actual transport concerns such as:

- `HttpClient`
- upstream HTTP requests
- HTTP status codes
