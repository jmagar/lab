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

Every failure returns JSON with a stable `kind` tag. Dispatcher-layer kinds (wrap `ApiError::kind()` tags from the SDK):

| `kind` | When |
|--------|------|
| `unknown_action` | action not in the service's action table. Include `valid: [...]` and fuzzy `hint`. |
| `unknown_subaction` | subaction segment invalid. |
| `missing_param` | required param absent. Include `param` name. |
| `invalid_param` | param present but wrong type/value. |
| `unknown_instance` | multi-instance label not found. Include `valid: [...]`. |

SDK-layer kinds (pass through from `ApiError::kind()`): `auth_failed`, `not_found`, `rate_limited` (+ `retry_after_ms`), `validation_failed`, `network_error`, `server_error`, `decode_error`, `internal_error`.

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
