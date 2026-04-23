# Deploy

This document covers the device-runtime deployment model introduced by `lab serve`.

## Topology

One device in the fleet is named `master`. Every other fleet member points at it with:

```toml
[device]
master = "tootie"
```

The master hosts:

- `/mcp`
- `/v1/{service}`
- `/v1/gateway`
- Web UI
- `/v1/fleet/ws`
- `/v1/device/*` fleet ingest and query routes

Non-master devices host:

- `/health`
- `/ready`
- `/v1/device/*`

They do not expose the operator control plane.

## Master Setup

1. choose a stable hostname for the master
2. set `[device].master` to that hostname on every non-master machine
3. start `lab serve --transport http` on the master
4. if the master binds beyond loopback, configure auth before startup

Example:

```bash
LAB_MCP_TRANSPORT=http
LAB_MCP_HTTP_HOST=0.0.0.0
LAB_MCP_HTTP_PORT=8765
LAB_MCP_HTTP_TOKEN=replace-me
lab serve --transport http
```

## Non-Master Setup

Each non-master device needs:

- the same `[device].master` value
- HTTP reachability to `http://<master>:<port>`
- fleet websocket reachability to `ws://<master>:<port>/v1/fleet/ws` or `wss://...`

Example `config.toml`:

```toml
[device]
master = "tootie"

[mcp]
port = 8765
```

Then start:

```bash
lab serve --transport http
```

The non-master runtime will queue metadata and bootstrap logs locally, open its fleet websocket session automatically, and keep retrying until the master admits the device.

## Enrollment Rollout

New or rotated devices are not admitted automatically.

Expected rollout:

1. deploy the master first
2. start or restart non-master devices
3. list pending enrollments on the master
4. approve or deny devices explicitly

Examples:

```bash
lab device enrollments list
lab device enrollments approve dookie
lab device enrollments deny steamy-wsl --reason "unexpected token"
```

## Operational Notes

- if `[device].master` is omitted, the local machine resolves as `master`
- fleet device identity is pinned by the enrollment store, not by the shared bearer token
- fleet inventory and logs are stored in memory on the master in this implementation
- enrollment records survive master restarts
- Web UI requests to a non-master return `403` with a clear disabled message

## Verification

Useful checks after deployment:

```bash
curl http://<device>:8765/health
curl -H "Authorization: Bearer $LAB_MCP_HTTP_TOKEN" http://<master>:8765/v1/device/devices
curl -H "Authorization: Bearer $LAB_MCP_HTTP_TOKEN" http://<master>:8765/v1/device/enrollments
lab device list
lab device enrollments list
lab logs search <device> <query>
```
