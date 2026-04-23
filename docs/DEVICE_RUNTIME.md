# Device Runtime

`lab serve` is now the always-on device runtime for every Linux `x86_64` machine that participates in a `lab` fleet.

One machine is the configured `master`. It owns the operator control plane:

- Web UI
- MCP
- `/v1/{service}` REST routes
- `/v1/gateway`
- device fleet inventory
- device log ingestion and search

Every other machine runs as a non-master device. Non-master devices keep only the local runtime and the `/v1/device/*` namespace.

## Role Resolution

Device role is resolved from:

1. local hostname
2. `[device].master` in `config.toml`, when present

If `[device].master` is missing, the local host resolves itself as `master`.

Example:

```toml
[device]
master = "tootie"
```

On `tootie`, `lab serve` runs as the master. On any other host, it runs as a non-master device and reports to `tootie`.

## Startup Behavior

When `lab serve` starts, it resolves the local hostname and device role, creates the in-process device runtime, and then:

- on the master:
  - creates the shared fleet state store
  - creates the durable enrollment store at `~/.lab/device-enrollments.json`
  - mounts the full operator control plane plus `/v1/device/*`
  - exposes the fleet websocket endpoint at `GET /v1/fleet/ws`
- on a non-master:
  - mounts `/health`, `/ready`, and `/v1/device/*`
  - disables the Web UI, MCP, gateway management, and the service REST surface
  - scans local MCP config inventory and queues metadata
  - collects bootstrap logs and queues them locally
  - starts a long-lived websocket session to the master and drains metadata, status, and log envelopes over that connection

## Fleet Transport

Non-master devices now use a websocket-first fleet transport:

1. derive `ws://` or `wss://` from the configured master base URL
2. connect to `GET /v1/fleet/ws`
3. send `initialize` with:
   - `lab.device_id`
   - `lab.device_token`
   - `lab.tailnet_identity`
4. wait for enrollment approval
5. once approved, keep the socket open and send:
   - `fleet/metadata.push`
   - `fleet/status.push`
   - `fleet/log.event`

Unknown devices are rejected at `initialize` and recorded as pending enrollments on the master. Operators approve or deny them through the master API, CLI, or MCP surface.

## Device API Namespace

The device runtime still exposes `/v1/device/*`, but websocket fleet delivery is now the canonical device-to-master path.

Write-oriented routes:

- `POST /v1/device/oauth/relay/start`
- `POST /v1/device/enrollments/{device_id}/approve`
- `POST /v1/device/enrollments/{device_id}/deny`

Read-oriented routes:

- `GET /v1/device/enrollments`
- `GET /v1/device/devices`
- `GET /v1/device/devices/{device_id}`
- `POST /v1/device/logs/search`

Fleet read routes are master-only. On a non-master device they return a structured `not_found` error rather than exposing an empty or partial local view.

## Metadata Inventory

Non-master devices scan the current home directory for MCP config inventory from:

- `~/.claude.json`
- `~/.codex/config.toml`
- `~/.gemini/settings.json`

Each discovered file is reported with:

- source name
- path
- modified timestamp
- SHA-256 content hash
- parsed MCP server map

This is inventory only. The master stores the uploaded metadata in memory for fleet inspection.

## Log Buffering

Device outbound delivery uses a durable local queue rooted at:

```text
~/.lab/device-runtime-queue.jsonl
```

Rules:

- non-master devices append metadata, status, and log envelopes locally first
- the queue is acknowledged only after the master accepts each websocket request
- failed uploads remain on disk for the next flush attempt
- the master stores ingested normalized log events in its in-process fleet store

The current bootstrap collector is intentionally minimal. It normalizes into the shared `DeviceLogEvent` shape and is expected to grow without changing the device API contract.

## Enrollment

Enrollment is master-owned and durable.

- pending, approved, and denied enrollment records are stored in `~/.lab/device-enrollments.json`
- approval pins an exact `(device_id, device_token)` pair
- denied devices remain blocked until explicitly re-approved with a new pending record

Operator surfaces:

- CLI:
  - `lab device enrollments list`
  - `lab device enrollments approve <device_id> [--note ...]`
  - `lab device enrollments deny <device_id> [--reason ...]`
- HTTP:
  - `GET /v1/device/enrollments`
  - `POST /v1/device/enrollments/{device_id}/approve`
  - `POST /v1/device/enrollments/{device_id}/deny`
- MCP:
  - `device.enrollments.list`
  - `device.enrollments.approve`
  - `device.enrollments.deny`

## OAuth Relay Capability

The device runtime exposes the existing local OAuth relay helper through:

```http
POST /v1/device/oauth/relay/start
```

Request body:

```json
{
  "bind_addr": "127.0.0.1:38935",
  "target_url": "http://100.88.16.79:38935/callback/dookie",
  "default_port": 38935,
  "request_timeout_ms": 30000
}
```

This starts the same local loopback forwarder used by `lab oauth relay-local`, but initiated through the device runtime on the target machine.

## Auth Expectations

When `LAB_MCP_HTTP_TOKEN` is configured, the master still protects operator `/v1/*` routes and master-routed CLI traffic with that bearer token.

Device-to-master fleet delivery itself does not depend on bearer auth anymore. It is authenticated inside websocket `initialize` using the device token pinned in the enrollment store.

OAuth mode still protects the public operator surface. Device fleet sessions do not mint or refresh OAuth credentials on their own.
