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
  - mounts the full operator control plane plus `/v1/device/*`
- on a non-master:
  - mounts `/health`, `/ready`, and `/v1/device/*`
  - disables the Web UI, MCP, gateway management, and the service REST surface
  - sends an initial hello to the master
  - uploads discovered AI CLI MCP config inventory
  - collects bootstrap logs, queues them locally, and attempts one flush to the master

## Device API Namespace

The device runtime uses `/v1/device/*` for fleet traffic and fleet queries.

Write-oriented routes:

- `POST /v1/device/hello`
- `POST /v1/device/status`
- `POST /v1/device/metadata`
- `POST /v1/device/syslog/batch`
- `POST /v1/device/oauth/relay/start`

Read-oriented routes:

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

Device log uploads use a durable JSONL queue at:

```text
~/.lab/device-runtime-queue.jsonl
```

Rules:

- non-master devices append outbound log batches locally first
- the queue is acknowledged only after the master accepts the batch
- failed uploads remain on disk for the next flush attempt
- the master stores ingested normalized log events in its in-process fleet store

The current bootstrap collector is intentionally minimal. It normalizes into the shared `DeviceLogEvent` shape and is expected to grow without changing the device API contract.

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

When `LAB_MCP_HTTP_TOKEN` is configured, master-bound device HTTP requests reuse that bearer token. This is the supported authentication path for device-to-master traffic in this implementation.

OAuth mode still protects the public operator surface, but device-to-master background uploads do not yet mint or refresh OAuth credentials on their own.
