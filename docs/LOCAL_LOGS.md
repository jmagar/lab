# Local Logs

`lab` now has a dedicated local-master runtime log subsystem.

This is separate from fleet device log search:

- fleet device log search remains `lab logs search <device> <query>` and `POST /v1/device/logs/search`
- local-master runtime logs live in the shared `dispatch::logs` subsystem and power `lab logs local *`, MCP `logs.*`, `POST /v1/logs`, `GET /v1/logs/stream`, and the gateway-admin `/logs` page

## Scope

Included:

- persistent indexed local history across restarts
- bounded search and tail access from CLI, MCP, and API
- true live streaming over HTTP SSE
- one shared normalized event model across adapters
- fleet-wide peer syslog ingestion (v2 — see [Fleet Syslog](#fleet-syslog-ingestion) below)

Not included:

- long-lived MCP streaming

## Surface Map

CLI:

- `lab logs local search`
- `lab logs local tail`
- `lab logs local stats`
- `lab logs local stream` — rejected with guidance; use `GET /v1/logs/stream` or `lab logs local tail`
- `lab logs forward` — peer syslog forwarder; reads journald or `/var/log/syslog` and batches to master

MCP:

- `logs.search`
- `logs.tail`
- `logs.stats`

HTTP:

- `POST /v1/logs`
- `GET /v1/logs/stream`
- `POST /v1/logs/ingest` — peer batch endpoint; requires `LAB_LOGS_INGEST_ENABLED=true` on master

Web UI:

- gateway-admin `/logs`

## Runtime Model

The shared `LogSystem` runtime owns:

- the bounded ingest queue
- normalization and redaction
- the embedded SQLite-backed store
- retention enforcement
- the in-process live subscriber hub used by SSE

`main.rs` owns tracing setup and attaches the ingest layer once. `lab serve` bootstraps the long-lived `LogSystem` runtime once and wires it into the shared HTTP/MCP state. One-shot CLI commands open the same on-disk store for bounded queries instead of constructing their own partial live runtime.

## Query And Streaming Rules

- historical search is store-backed
- `logs.tail` is bounded follow-up access, not true streaming
- true live streaming is HTTP SSE only
- hosted same-origin browser session auth is the supported v1 SSE access mode
- standalone bearer mode does not get implicit browser EventSource support in v1

## Retention

Retention is enforced by the shared subsystem and is whichever limit hits first:

- `retention_days`
- `max_bytes`

Oldest events are evicted first.

Operators can inspect retention state through:

- `lab logs local stats`
- `logs.stats`
- `POST /v1/logs` with `{"action":"logs.stats"}`
- the gateway-admin `/logs` page

## Configuration

`[local_logs]` in `config.toml` or env overrides:

- `LAB_LOCAL_LOGS_STORE_PATH`
- `LAB_LOCAL_LOGS_RETENTION_DAYS`
- `LAB_LOCAL_LOGS_MAX_BYTES`
- `LAB_LOCAL_LOGS_QUEUE_CAPACITY`
- `LAB_LOCAL_LOGS_SUBSCRIBER_CAPACITY`

See [CONFIG.md](./CONFIG.md) for the canonical key table.

## Redaction

Redaction happens before persistence and before live fanout.

The local store and SSE stream must not expose:

- bearer tokens
- cookies
- authorization headers
- raw OAuth token material
- secret env values

See [OBSERVABILITY.md](./OBSERVABILITY.md) for the canonical redaction policy.

## Fleet Syslog Ingestion

All peer nodes (running the same `lab` binary) can forward their local syslog to the master.

### Master config

Enable the ingest endpoint on the master:

```bash
LAB_LOGS_INGEST_ENABLED=true
```

Auth uses the same shared bearer token already in use for `lab serve`:

```bash
LAB_MCP_HTTP_TOKEN=<token>
```

### Peer config

On each peer node:

```bash
LAB_MASTER_URL=http://<master-host>:<port>   # required
LAB_MASTER_TOKEN=<token>                     # falls back to LAB_MCP_HTTP_TOKEN
LAB_NODE_ID=<hostname-or-label>              # falls back to $(hostname)
```

### Running the forwarder

```bash
lab logs forward
```

Reads journald (`journalctl --follow --output=json`) by default, falls back to
`/var/log/syslog` if `journalctl` is not available, or if `--syslog-only` is passed.

Options:

```
--master-url     Override LAB_MASTER_URL
--master-token   Override LAB_MASTER_TOKEN
--node-id        Override LAB_NODE_ID
--batch-size     Events per POST (default 200)
--syslog-only    Skip journald, read /var/log/syslog directly
```

### Searching across nodes

The master's `LogQuery` now supports two new filter fields:

- `source_node_ids: string[]` — filter by originating node label
- `source_kinds: string[]` — filter by source type (`syslog`, `local`, …)

These are available via MCP `logs.search` and `POST /v1/logs`. The CLI does not yet expose `--source-node-id` or `--source-kind` flags; use MCP or the HTTP API to filter by source.

### Wire format

Peer events arrive via `POST /v1/logs/ingest` with:

```json
{ "node_id": "my-server", "events": [<RawLogEvent>] }
```

The master stamps `source_node_id` from `node_id` (overrides any self-reported field) and
routes every event through `canonicalize()` before persistence — redaction is always enforced.

### Reserved event fields

The normalized event model retains these fields for fleet/syslog provenance:

- `source_kind` — `"syslog"`, `"local"`, etc.
- `source_node_id` — the forwarding node label
- `source_device_id` — optional hardware/device identity
- `ingest_path` — `"journald"`, `"syslog_file"`, `"tracing"`, etc.
- `upstream_event_id` — cursor from the origin log stream (e.g. journald cursor)
