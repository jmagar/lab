# Deploy

This document covers the node-runtime deployment model introduced by `lab serve`.

## Topology

One node is the configured `controller`. Every other node points at it with:

```toml
[node]
controller = "tootie"
```

The controller hosts:

- `/mcp`
- `/v1/{service}`
- `/v1/gateway`
- Web UI
- `/v1/nodes/ws`
- `/v1/nodes/*` fleet ingest and query routes

Non-controller nodes host:

- `/health`
- `/ready`
- `/v1/nodes/*`

They do not expose the operator control plane.

## Master Setup

1. choose a stable hostname for the master
2. set `[device].master` to that hostname on every non-master machine
3. start `lab serve` on the master
4. if the master binds beyond loopback, configure auth before startup

Example:

```bash
LAB_MCP_TRANSPORT=http
LAB_MCP_HTTP_HOST=0.0.0.0
LAB_MCP_HTTP_PORT=8765
LAB_MCP_HTTP_TOKEN=replace-me
lab serve
```

## Non-Master Setup

Each non-master device needs:

- the same `[device].master` value
- HTTP reachability to `http://<master>:<port>`
- node websocket reachability to `ws://<master>:<port>/v1/nodes/ws` or `wss://...`

Example `config.toml`:

```toml
[node]
controller = "tootie"

[mcp]
port = 8765
```

Then start:

```bash
lab serve
```

The non-master runtime will queue metadata and bootstrap logs locally, open its node websocket session automatically, and keep retrying until the master admits the node.

## Enrollment Rollout

New or rotated devices are not admitted automatically.

Expected rollout:

1. deploy the master first
2. start or restart non-controller nodes
3. list pending enrollments on the master
4. approve or deny nodes explicitly

Examples:

```bash
lab nodes enrollments list
lab nodes enrollments approve dookie
lab nodes enrollments deny steamy-wsl --reason "unexpected token"
```

## Operational Notes

- if `[device].master` is omitted, the local machine resolves as `master`
- fleet node identity is pinned by the enrollment store, not by the shared bearer token
- node inventory and logs are stored in memory on the master in this implementation
- enrollment records survive master restarts
- Web UI requests to a non-master return `403` with a clear disabled message

## Verification

Useful checks after deployment:

```bash
curl http://<device>:8765/health
curl -H "Authorization: Bearer $LAB_MCP_HTTP_TOKEN" http://<master>:8765/v1/device/devices
curl -H "Authorization: Bearer $LAB_MCP_HTTP_TOKEN" http://<master>:8765/v1/device/enrollments
lab nodes list
lab nodes enrollments list
lab logs search <device> <query>
```
