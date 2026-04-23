# Fleet Logs

Fleet logs are normalized device log events ingested by the master from non-master device runtimes.

## Event Shape

Each log event uses the shared `DeviceLogEvent` schema:

- `device_id`
- `source`
- `timestamp_unix_ms`
- optional `level`
- `message`
- structured `fields`

This keeps fleet log ingestion independent from the raw source format.

## Ingestion Flow

1. a non-master device collects bootstrap log events
2. it appends a `syslog_batch` envelope to `~/.lab/device-runtime-queue.jsonl`
3. it sends the envelope over the live websocket session as `fleet/log.event`
4. the master stores the normalized events in the in-memory fleet store
5. the local queue entry is acknowledged only after a successful websocket response

The queue exists to make early-runtime log upload resilient to temporary master outages.

## Query Surfaces

Fleet log search is available on the master through:

- CLI: `lab logs search <device> <query>`
- HTTP: `POST /v1/device/logs/search`

Example:

```bash
lab logs search dookie oauth
```

```json
POST /v1/device/logs/search
{
  "device_id": "dookie",
  "query": "oauth"
}
```

The current implementation performs a case-insensitive substring match against `message`.

## Fleet Device Queries

The master also exposes:

- `lab device list`
- `lab device get <device_id>`
- `GET /v1/device/devices`
- `GET /v1/device/devices/{device_id}`

Those responses include per-device log counts so operators can quickly see whether a device is checking in and sending data.

## Current Limits

- fleet state is in-process only; restarting the master clears the inventory and ingested logs
- enrollment state is durable, but accepted metadata/status/log snapshots are still in-memory
- log search currently matches `message` only
- the bootstrap collector is intentionally conservative and may return no events on hosts without supported sources
