# Gateway Management

`lab` exposes a first-class `gateway` management surface for the upstream MCP proxy defined in [UPSTREAM.md](./UPSTREAM.md).

Use it when you want to inspect, test, add, update, remove, or reload `[[upstream]]` entries without editing `~/.config/lab/config.toml` by hand.

## Scope

- `[[upstream]]` in `~/.config/lab/config.toml` remains the persisted source of truth.
- `gateway.*` actions mutate that config, reconcile runtime state, and trigger MCP list-changed notifications when the merged catalog changes.
- In-flight MCP requests keep using the pool they already captured. New requests observe the swapped pool after reconcile completes.

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
{ "action": "gateway.update", "params": { "name": "github", "patch": { "expose_tools": ["search_repos", "github_*"] } } }
```

```json
{ "action": "gateway.update", "params": { "name": "github", "patch": { "expose_tools": null } } }
```

## Validation

- exactly one of `url` or `command` must be set
- `url` must use `http://` or `https://`
- bind-all addresses (`0.0.0.0`, `::`) are rejected
- RFC1918 and other private-network URLs are allowed
- stdio gateways are allowed

## Reconcile Model

Every mutating action follows the same sequence:

1. read and validate config
2. write `~/.config/lab/config.toml` with temp-file-in-same-dir plus rename
3. build a fresh upstream pool outside the config mutation lock
4. atomically swap the runtime handle
5. notify connected MCP peers when tool/resource/prompt catalogs changed

## Examples

### CLI

```bash
lab gateway list
lab gateway get remote-lab
lab gateway test --name remote-lab
lab gateway add --name remote-lab --url https://lab2.example.com/mcp --bearer-token-env REMOTE_LAB_TOKEN
lab gateway update remote-lab --proxy-resources true
lab gateway remove remote-lab
lab gateway reload
```

### MCP

```json
{ "tool": "gateway", "input": { "action": "gateway.list", "params": {} } }
{ "tool": "gateway", "input": { "action": "gateway.add", "params": { "spec": { "name": "remote-lab", "url": "https://lab2.example.com/mcp", "bearer_token_env": "REMOTE_LAB_TOKEN" } } } }
{ "tool": "gateway", "input": { "action": "gateway.reload", "params": {} } }
```

### HTTP API

```json
POST /v1/gateway
{ "action": "gateway.list", "params": {} }
```

```json
POST /v1/gateway
{ "action": "gateway.update", "params": { "name": "remote-lab", "patch": { "proxy_resources": true } } }
```

## Limitations

- `gateway.reload` is the only action that promises to pick up changed bearer-token env vars.
- The product HTTP API exposes `/v1/gateway` for gateway management, but it still does not proxy arbitrary upstream MCP tools through `/v1/*`.
- Runtime counts depend on current discovery state; an unreachable upstream can remain configured while reporting zero discovered items.
- Gateway mutations rewrite `config.toml` by serializing the full `LabConfig` struct. TOML comments and unknown keys not represented in the struct are dropped on write. A migration to `toml_edit` for comment-preserving round-trips is deferred.
