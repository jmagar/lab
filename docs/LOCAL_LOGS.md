# Local Logs

`lab` now has a dedicated local-master runtime log subsystem.

This is separate from fleet device log search:

- fleet device log search remains `lab logs search <device> <query>` and `POST /v1/device/logs/search`
- local-master runtime logs live in the shared `dispatch::logs` subsystem and power `lab logs local *`, MCP `logs.*`, `POST /v1/logs`, `GET /v1/logs/stream`, and the gateway-admin `/logs` page

## Scope

The v1 scope is the current master process only.

Included:

- persistent indexed local history across restarts
- bounded search and tail access from CLI, MCP, and API
- true live streaming over HTTP SSE
- one shared normalized event model across adapters

Not included:

- fleet-wide device aggregation
- remote syslog ingestion
- long-lived MCP streaming

## Surface Map

CLI:

- `lab logs local search`
- `lab logs local tail`
- `lab logs local stats`
- `lab logs local stream` — rejected with guidance; use `GET /v1/logs/stream` or `lab logs local tail`

MCP:

- `logs.search`
- `logs.tail`
- `logs.stats`

HTTP:

- `POST /v1/logs`
- `GET /v1/logs/stream`

Web UI:

- gateway-admin `/logs`

## Runtime Model

The shared `LogSystem` runtime owns:

- the bounded ingest queue
- normalization and redaction
- the embedded SQLite-backed store
- retention enforcement
- the in-process live subscriber hub used by SSE

`main.rs` and `lab serve` bootstrap the long-lived runtime once. One-shot CLI commands open the same on-disk store for bounded queries instead of constructing their own partial live runtime.

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

## Future Fleet And Syslog Seams

The normalized event model keeps these reserved fields intentionally:

- `source_kind`
- `source_node_id`
- `source_device_id`
- `ingest_path`
- `upstream_event_id`

These are not accidental over-abstraction.

They exist so future remote device and syslog ingestion can reuse the same store, search, and UI contracts without schema churn or a breaking API redesign.
