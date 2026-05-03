---
name: mcp-tool-checklist
description: Authoritative checklist for new or modified MCP tools — token-consciousness rules, tool registration shape, and the 4-element description template. Sourced from spec.md § MCP Conventions + § Plan Integrations → MCP additions. Preloaded by mcp-tool-reviewer.
user-invocable: false
---

Source of truth: `spec.md § MCP Conventions (token consciousness)` (L1966-2033) and `spec.md § Plan Integrations → MCP additions` (L4961-5022).

The discipline these rules enforce: agents talking to MCP have small context windows. Tools that return multi-MB blobs or untruncated logs poison the conversation. The framework handles most of this automatically via `respond()` — the reviewer's job is to verify the framework is being used and not bypassed.

---

## §1 — Pagination

Every `list`-style action MUST accept:
- `limit` — default `20`, max `100` (clamped, not rejected — out-of-range values are silently capped)
- `cursor` — opaque `Option<String>`

Response shape:
```json
{ "items": [...], "next_cursor": null | "...", "total": null | 1234 }
```

Cursor format is opaque to clients but should be deterministic (e.g. `base64(JSON({offset, sort_key}))`).

**Violations to flag:**
- `list` action with no `limit`/`cursor` params
- Returning all items without pagination
- Rejecting (vs clamping) out-of-range `limit`
- Cursor format that changes between identical inputs (breaks resumability)

## §2 — Field selection

All actions MUST accept optional `fields: ["id", "name", ...]`. Implemented as a post-serialization filter in `core::respond()` — drops keys not in the list. Special value `"*"` returns all (default).

**Violations to flag:**
- Action that ignores the `fields` parameter (skips the filter)
- Action that hand-rolls field filtering instead of going through `respond()`

## §3 — Smart truncation

String fields larger than **2KB** must be replaced with the truncation envelope:

```json
{
  "_truncated": true,
  "preview":     "first 500 chars...",
  "full_action": "item.get_content",
  "full_params": { "id": "..." }
}
```

Truncation is opt-in per field via a `#[truncate]` attribute on the domain type (custom serde wrapper).

**Violations to flag:**
- Domain type with multi-KB string fields and no `#[truncate]` attribute
- Truncation envelope missing `full_action` or `full_params` (agent can't fetch the full value)
- `preview` longer than 500 chars or shorter than is useful

## §4 — Output budget guard

After serialization, if response > **8KB**, `respond()` strips non-essential fields (bodies, descriptions, raw payloads), keeps IDs/names/timestamps/status, and sets:

```json
{
  "_overflow":        true,
  "_overflow_action": "item.get",
  ...
}
```

Implemented once in `core::respond(action: &str, value: Value) -> Value`. Every handler in `run_with_client` MUST end with `Ok(respond(action, v))`.

**Violations to flag:**
- Handler that returns a `Value` directly without going through `respond()`
- Custom truncation logic in a handler (bypasses the central guard)
- `_overflow_action` pointing to a non-existent or wrong action

## §5 — Tool registration

- **One MCP tool per domain.** Tool name = domain name (`example`, not `example_item_list`)
- Schema MUST be: `{ action: string, params?: object, fields?: string[], limit?: number, cursor?: string }`
- Tool description embeds the rendered `help` output (action list with one-line descriptions) so agents see the full catalog without a separate call
- `destructive: true` on `ActionSpec` triggers `peer.elicit::<Confirm>(...)` before executing

**Violations to flag:**
- Multiple MCP tools per domain (e.g. `example_list`, `example_get`)
- Schema deviating from the canonical 5-field shape
- Tool description without the embedded help output
- Destructive `ActionSpec` without elicitation wired
- New action with `destructive: true` but no clear matching `idempotent` value

## §6 — Transport

Supported: **stdio** (default for local agents), **Streamable HTTP** (remote agents).

**Not supported: SSE.** Deprecated in MCP spec. `rmcp` keeps it; we don't ship it.

**Violations to flag:**
- New transport wiring that exposes SSE
- stdio MCP lacking the panic-isolation wrapper (see §8)
- Streamable HTTP without auth middleware integration

## §7 — `request_id` in success envelope

The `respond()` helper injects `_meta.request_id` into every success value. The MCP wrapper preserves it in the JSON-string content:

```json
{
  "isError": false,
  "content": [{ "type": "text", "text": "{\"items\":[...],\"_meta\":{\"request_id\":\"...\"}}" }]
}
```

**Violations to flag:**
- Success envelope without `_meta.request_id`
- MCP wrapper that strips `_meta` before returning
- Hand-built MCP responses that don't go through `respond()`

## §8 — stdio panic isolation

Each tool dispatch MUST be wrapped in `futures::FutureExt::catch_unwind`:

```rust
async fn dispatch_tool(params: Value) -> Result<Value, ToolError> {
    use futures::FutureExt;
    let result = std::panic::AssertUnwindSafe(actual_dispatch(params))
        .catch_unwind()
        .await;
    match result {
        Ok(r)  => r,
        Err(_) => {
            tracing::error!(target: "panic", "tool handler panicked");
            Err(ToolError::Internal { message: "tool handler panicked".into() })
        }
    }
}
```

Verify rmcp 1.5 doesn't already do this — if it does, the wrapper is redundant. As of spec date, it does not.

**Violations to flag:**
- Tool dispatch without `catch_unwind` wrapping
- Panic message leaked into `ToolError::Internal` (must be generic "tool handler panicked")
- `panic` log event missing or wrong `target`

## §9 — Tool description: 4-element template

Per spec (citing arXiv 2602.14878: 97.1% of MCP tools have description deficiencies), descriptions follow:

```
{domain}: {one-sentence purpose}.

When to use: {activation criteria}.
Parameters:
  - action (required): {subject.verb pattern}. Call {action:"help"} for the full catalog.
  - params (optional): action-specific. Call {action:"schema", params:{action:"X"}} for shape.
  - fields, limit, cursor: standard pagination/projection.
Limitations: {rate limits, destructive-action elicitation rules, known caveats}.
```

Generated by `xtask gen-tool-descriptions` from each domain's `Description` static and `ACTIONS` catalog. Drift caught by `check-tool-descriptions-drift`.

**Violations to flag:**
- Description missing any of the four elements (purpose, when-to-use, parameters, limitations)
- Hand-edited description that differs from what `gen-tool-descriptions` would produce
- "Parameters" section that doesn't reference `help` and `schema` actions

---

## Output format

Group findings by section number with `file:line` refs:

```
=== §1: Pagination ===
OK

=== §3: Smart truncation ===
crates/core/src/example/types/item.rs:18  field `body: String` >2KB potential, missing #[truncate]

=== §4: Output budget guard ===
crates/core/src/example/handlers.rs:147  handler returns Value without respond() — bypasses budget guard

...
```

End with `{n} violations, {n} sections OK`. Verdict: **PASS** / **PASS with warnings** / **FAIL**.
