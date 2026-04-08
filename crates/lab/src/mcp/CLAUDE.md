# mcp/ — MCP protocol surface

This directory is the translation layer between `lab-apis` (pure SDK) and the MCP protocol. It owns dispatch, envelopes, resources, elicitation, and the shared catalog.

## One tool per service

Each enabled service registers exactly one MCP tool in `registry.rs`. The tool name matches the service name (`radarr`, `sonarr`, `extract`, ...). Registration is feature-gated:

```rust
#[cfg(feature = "radarr")]
registry.register("radarr", services::radarr::dispatch);
```

## Dispatch pattern

`services/<service>.rs` is a thin dispatcher that:

1. Reads `params.action` (required).
2. Reads `params.instance` (optional, defaults to main; returns `unknown_instance` envelope if label missing).
3. Matches on action string → calls the corresponding `FooClient` method with typed params extracted from `params`.
4. Wraps the result in `ToolEnvelope<T>` or a `ToolError` envelope.

**No business logic here.** If you find yourself calling `reqwest`, parsing JSON beyond param extraction, or retrying, you're in the wrong crate — move it to `lab-apis/src/<service>/client.rs`.

## Structured error envelopes

`ToolError` in `envelope.rs` is the **single canonical error type** across all three surfaces — MCP, HTTP API, and CLI. Every failure returns the same JSON shape:

```jsonc
{ "kind": "missing_param", "message": "missing required parameter `query`", "param": "query" }
{ "kind": "unknown_action", "message": "...", "valid": ["movie.list", ...], "hint": null }
{ "kind": "auth_failed",    "message": "authentication failed" }   // SDK pass-through
```

Dispatcher-layer kinds:

| `kind` | When |
|--------|------|
| `unknown_action` | action not in the service's action table. Include `valid: [...]` and fuzzy `hint`. |
| `unknown_subaction` | subaction segment invalid. |
| `missing_param` | required param absent. Include `param` name. |
| `invalid_param` | param present but wrong type/value. |
| `unknown_instance` | multi-instance label not found. Include `valid: [...]`. |

SDK-layer kinds pass through from `ApiError::kind()` via `From<SdkError> for ToolError`: `auth_failed`, `not_found`, `rate_limited`, `validation_failed`, `network_error`, `server_error`, `decode_error`, `internal_error`.

### Serialization contract

`ToolError` uses a **custom `Serialize`** (not `#[derive(Serialize)]`) so that the `Sdk` variant promotes its `sdk_kind` field to the top-level `kind` field. The result is byte-identical across MCP and HTTP — never `{"kind":"sdk","sdk_kind":"auth_failed"}`.

- `Display` delegates to `serde_json::to_string(&self)` — output is always valid JSON.
- `IntoResponse` serializes `self` directly; HTTP status is derived from `kind()`.
- Tests in `envelope.rs` lock in this contract — do not break them.

### Wiring per service

Each service dispatcher must:
1. Return `Result<Value, ToolError>` (not `anyhow::Result`).
2. Implement `From<ServiceError> for ToolError` mapping via `ApiError::kind()`.
3. Use `ToolError::MissingParam` / `UnknownAction` for dispatcher-layer errors.
4. Never use `anyhow::bail!` or `anyhow::anyhow!` inside a dispatch function.

## Elicitation for destructive ops

When an action's `ActionSpec.destructive == true`, the dispatcher **must** call the MCP elicitation flow before executing. The client confirms, then the dispatcher proceeds. Do not bypass this even if the caller passed a "confirm" param — elicitation is the protocol contract.

## Built-in actions

Every tool automatically supports `help` and `schema` without the service declaring them. The dispatcher intercepts these before the action match.

## Shared catalog — one builder, three surfaces

`build_catalog()` (in `lab/src/catalog.rs` or similar) is the **single source** feeding:

1. The `lab.help` global MCP tool.
2. The `lab://catalog` MCP resource.
3. The `lab help` CLI subcommand.

Never duplicate catalog logic. If you need richer data, extend the builder.

## Resources

- `lab://<service>/actions` — per-service action catalog (name, description, destructive, params).
- `lab://catalog` — the full cross-service catalog.

Resources are read-only. Do not use them for mutations.
