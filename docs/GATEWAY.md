# Gateway Management

`lab` exposes a first-class `gateway` management surface for the upstream MCP proxy defined in [UPSTREAM.md](./UPSTREAM.md).

This is separate from the device runtime `master` model. `gateway` remains the upstream MCP control plane and must not be overloaded for fleet device identity, device ingest, or fleet log handling.

Use it when you want to inspect, test, add, update, remove, or reload `[[upstream]]` entries without editing `~/.config/lab/config.toml` by hand.

## Scope

- `[[upstream]]` in `~/.config/lab/config.toml` remains the persisted source of truth.
- `gateway.*` actions mutate that config, reconcile runtime state, and trigger MCP list-changed notifications when the merged catalog changes.
- In-flight MCP requests keep using the pool they already captured. New requests observe the swapped pool after reconcile completes.
- gateway management is exposed on the `master` only; non-master devices do not mount `/v1/gateway` or the `/mcp` transport

Secrets remain indirect:

- config and management responses may include `bearer_token_env`
- token values are never returned
- token values are never written into TOML
- changing an env var alone does not hot-apply; call `gateway.reload`
- tool exposure filters are stored as names/patterns only; the gateway never rewrites upstream tool payloads

## Actions

| Action | Purpose |
|-------|---------|
| `gateway.list` | Return all configured gateways with config and runtime views. |
| `gateway.get` | Return one configured gateway. |
| `gateway.test` | Probe a configured gateway by name or a proposed unsaved spec. |
| `gateway.add` | Persist a new `[[upstream]]` entry, reconcile runtime state, return the new view. |
| `gateway.update` | Patch an existing gateway, reconcile runtime state, return the new view. |
| `gateway.remove` | Remove a gateway, reconcile runtime state, return the removed config view. |
| `gateway.reload` | Reload `~/.config/lab/config.toml`, rebuild the runtime pool, and pick up changed env-token values. |
| `gateway.status` | Return runtime counts for configured gateways. |
| `gateway.discovered_tools` | Return discovered tool metadata and exposure state for one gateway. |
| `gateway.discovered_resources` | Return proxied resource URIs discovered for one gateway. |
| `gateway.discovered_prompts` | Return prompt names discovered for one gateway. |

`gateway.add`, `gateway.update`, `gateway.remove`, and `gateway.reload` are
destructive actions in shared action metadata. HTTP callers must send
`params.confirm = true`, CLI callers must confirm interactively or use `--yes` / `-y`, and MCP callers
must go through elicitation when supported.

### Stdio Gateway Safety

Stdio upstreams are privileged because testing or reconciling them starts the
configured command on the local host running `lab`. The gateway admin surface
therefore requires an additional explicit acknowledgement before any stdio
definition is probed or reconciled:

- `gateway.test` with a `spec.command`, or with `name` pointing at a configured
  stdio gateway, requires `params.allow_stdio = true`
- `gateway.add` with `spec.command` requires `params.allow_stdio = true`
- `gateway.update` requires `params.allow_stdio = true` whenever the resulting
  enabled gateway config uses `command`, even if the patch only changes
  unrelated fields

This acknowledgement is separate from destructive confirmation. `confirm: true`
authorizes config mutation; `allow_stdio: true` acknowledges local command
execution. HTTP and MCP callers should only send it after operator approval.

CLI commands expose the same guard as `--allow-stdio`.

## Tool Exposure

Gateway config can optionally restrict which discovered upstream tools are republished by `lab`.

- when `expose_tools` is unset, all discovered upstream tools remain exposed
- `expose_tools` accepts exact tool names and simple `*` wildcards
- an empty allowlist is treated as "clear the filter" rather than "block everything"
- filtered tools disappear from merged MCP `list_tools()` results and cannot be called directly through the proxy

Example:

```toml
[[upstream]]
name = "github"
url = "https://github.example.com/mcp"
bearer_token_env = "GITHUB_MCP_TOKEN"
proxy_resources = false
expose_tools = ["search_repos", "github_*"]
```

Typical patch payloads:

```json
{ "action": "gateway.update", "params": { "confirm": true, "name": "github", "patch": { "expose_tools": ["search_repos", "github_*"] } } }
```

```json
{ "action": "gateway.update", "params": { "confirm": true, "name": "github", "patch": { "expose_tools": null } } }
```

## Tool Search Mode

Gateway tool-search mode is a single gateway-wide switch. It is not configured per upstream server.

When enabled, Lab hides raw proxied upstream tools from MCP `list_tools()` and exposes two synthetic tools instead:

| Tool | Purpose |
|------|---------|
| `tool_search` | Search healthy discovered upstream tools across the gateway. |
| `tool_invoke` | Invoke one tool returned by `tool_search`. |

This keeps the MCP catalog small while still allowing clients to reach every exposed upstream tool. Per-upstream `expose_tools` filters still apply before tools enter the searchable catalog.

Configuration lives at root `[tool_search]` in `config.toml`:

```toml
[tool_search]
enabled = true
top_k_default = 10
max_tools = 5000
```

CLI:

```bash
lab gateway tool-search status
lab gateway tool-search enable
lab gateway tool-search enable --top-k-default 20 --max-tools 8000
lab gateway tool-search disable
```

HTTP/MCP gateway management actions:

```json
{ "action": "gateway.tool_search.get", "params": {} }
```

```json
{ "action": "gateway.tool_search.set", "params": { "enabled": true, "top_k_default": 10, "max_tools": 5000 } }
```

Search call shape on the MCP surface:

```json
{ "query": "github issue search", "top_k": 10, "include_schema": false }
```

Invoke call shape on the MCP surface:

```json
{ "name": "search_issues", "arguments": { "query": "repo:jmagar/lab tool_search" } }
```

Rules:

- `top_k_default` is validated in the range `1..=50`
- `max_tools` is validated in the range `1..=10000`
- `query` must be non-empty and no longer than 500 characters
- `include_schema` defaults to `false`; schemas are sanitized before return when requested
- old `[[upstream]].tool_search` blocks are accepted only as migration input and are dropped on the next gateway config write
- `gateway.update` rejects `patch.tool_search`; use `gateway.tool_search.set` instead

## Validation

- exactly one of `url` or `command` must be set
- `url` must use `http://` or `https://`
- bind-all addresses (`0.0.0.0`, `::`) are rejected
- RFC1918 and other private-network URLs are allowed
- stdio gateways are allowed only as an explicit privileged operator action.
  Proposed or persisted enabled stdio specs can execute local commands during
  `gateway.test`, `gateway.add`, and `gateway.update`. Machine callers must
  pass `allow_stdio: true` in addition to normal destructive confirmation where
  applicable; without that acknowledgement the request fails with
  `kind: "invalid_param"` on `allow_stdio`.

## Reconcile Model

Every mutating action follows the same sequence:

1. read and validate config
2. write `~/.config/lab/config.toml` with temp-file-in-same-dir plus rename
3. build a fresh upstream pool outside the config mutation lock
4. atomically swap the runtime handle
5. notify connected MCP peers when tool/resource/prompt catalogs changed

Observability requirements for that reconcile:

- log intent before config mutation begins
- log each phase transition (`config_write`, `pool_build`, `swap`, `notify`)
- log outcome with success/failure and elapsed time
- redact credential-bearing URLs, commands, args, and token-derived values in
  both logs and returned management views

## Examples

### CLI

```bash
lab gateway list
lab gateway get remote-lab
lab gateway test --name remote-lab
lab gateway add --name remote-lab --url https://lab2.example.com/mcp --bearer-token-env REMOTE_LAB_TOKEN
lab gateway add --name local-tools --command local-mcp-server --allow-stdio
lab gateway update remote-lab --proxy-resources true
lab gateway remove remote-lab
lab gateway reload
```

### MCP

```json
{ "tool": "gateway", "input": { "action": "gateway.list", "params": {} } }
{ "tool": "gateway", "input": { "action": "gateway.add", "params": { "confirm": true, "spec": { "name": "remote-lab", "url": "https://lab2.example.com/mcp", "bearer_token_env": "REMOTE_LAB_TOKEN" } } } }
{ "tool": "gateway", "input": { "action": "gateway.add", "params": { "confirm": true, "allow_stdio": true, "spec": { "name": "local-tools", "command": "local-mcp-server" } } } }
{ "tool": "gateway", "input": { "action": "gateway.reload", "params": { "confirm": true } } }
```

### HTTP API

```json
POST /v1/gateway
{ "action": "gateway.list", "params": {} }
```

```json
POST /v1/gateway
{ "action": "gateway.update", "params": { "confirm": true, "name": "remote-lab", "patch": { "proxy_resources": true } } }
```

## Upstream OAuth Routes

For upstreams configured with `[upstream.oauth]` (see
[CONFIG.md](./CONFIG.md#upstream-oauth-authorization_code--pkce) and
[UPSTREAM.md](./UPSTREAM.md#upstream-oauth-authorization_code--pkce)), the
gateway mounts four master-only HTTP routes. All four require an authenticated
session and the master-only middleware; non-master sessions get `403`.

| Method | Path | Purpose |
|--------|------|---------|
| `POST` | `/v1/gateway/oauth/start` | Begin authorization for the shared gateway subject `gateway`. Body `{ "upstream": "<name>" }`. Returns `{ "authorization_url": "..." }` (JSON only — no browser-redirect mode). |
| `GET` | `/auth/upstream/callback` | Authorization-code callback. Validates the authenticated session, atomically takes the pending state row (bound to `(upstream, subject)`), exchanges the code, persists encrypted credentials, redirects to `/gateway/oauth/result?upstream=<name>&status=<ok\|fail>`. |
| `GET` | `/v1/gateway/oauth/status?upstream=<name>` | Returns `{ "authenticated": bool, "upstream": "<name>", "expires_within_5m": bool }`. Deliberately omits subject and raw expiry timestamp to avoid enumeration and fingerprinting. |
| `POST` | `/v1/gateway/oauth/clear?upstream=<name>&confirm=true` | Destructive. Requires both `upstream` (the upstream name) and `confirm=true` as query parameters. Without `confirm=true`, returns `422` with JSON `{ "kind": "confirmation_required", ... }`. With confirm, deletes persisted credentials and evicts the cached `AuthClient`. In-flight requests complete naturally under the old credential (graceful drain by Rust ownership — not a designed protocol). |

### OAuth Operator Examples

CLI:

```bash
lab gateway mcp auth start chrome-devtools
lab gateway mcp auth open chrome-devtools --wait
lab gateway mcp auth status chrome-devtools
lab gateway mcp auth clear chrome-devtools
```

MCP tool calls:

```json
{ "tool": "gateway", "input": { "action": "gateway.oauth.start", "params": { "upstream": "chrome-devtools" } } }
{ "tool": "gateway", "input": { "action": "gateway.oauth.status", "params": { "upstream": "chrome-devtools" } } }
{ "tool": "gateway", "input": { "action": "gateway.oauth.clear", "params": { "confirm": true, "upstream": "chrome-devtools" } } }
```

These actions now operate on the shared gateway OAuth subject `gateway`, so the
web UI, CLI, and MCP tool surface all refer to the same stored upstream
credential row.

Callback security invariants (enforced in code, spec-required):

- The callback is a browser-facing redirect endpoint. Subject is resolved from
  the authenticated browser session cookie, **not** from the `state` parameter
  or the pending state row. No session → `oauth_state_invalid`.
- The `upstream` query parameter is forwarded to the manager, which enforces it
  against the pending state row's upstream name via the SQL primary key
  (`(upstream_name, subject, csrf_token)`).
- `state` is matched via a single `DELETE ... RETURNING` to prevent replay
  across connection-pool races.
- The result page HTML-escapes the operator-controlled `upstream` name.

### Reload And Credential Lifecycle

- `gateway.reload` eagerly evicts all cached `AuthClient` entries for every
  OAuth upstream in the current config, then rebuilds a fresh upstream pool.
  It does **not** delete persisted credential rows — `AuthClient`s are rebuilt
  on the next request using whatever credentials are in the store.
- `clear_credentials` is the only way to invalidate a persisted credential.
  It evicts the cache entry and deletes the row; in-flight `Arc<AuthClient>`
  holders complete naturally under the old token.
- Expired access-only credential rows (no refresh token) are pruned by the
  60-second `cleanup_expired` background task, alongside expired PKCE state.

## Limitations

- `gateway.reload` is the only action that promises to pick up changed bearer-token env vars.
- The product HTTP API exposes `/v1/gateway` for gateway management, but it still does not proxy arbitrary upstream MCP tools through `/v1/*`.
- Runtime counts depend on current discovery state; an unreachable upstream can remain configured while reporting zero discovered items.
- Gateway mutations rewrite `config.toml` by serializing the full `LabConfig` struct. TOML comments and unknown keys not represented in the struct are dropped on write. A migration to `toml_edit` for comment-preserving round-trips is deferred.
