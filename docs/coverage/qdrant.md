# Qdrant API Coverage

**Last updated:** 2026-04-14
**OpenAPI spec:** docs/upstream-api/qdrant.openapi.json
**OpenAPI version:** 3.0.1
**API version:** master
**Paths:** 73
**Servers:** {protocol}://{hostname}:{port}
**Security schemes:** api-key, bearerAuth

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and wired through SDK, dispatch, CLI, MCP, and API |
| ⬜ | Not implemented yet; rows are spec inventory only |
| - | Not applicable / not represented in the spec |

The source spec is the contract. This document is the implementation planning aid.

## Dispatch Actions

The following actions are implemented end-to-end (SDK → dispatch → MCP → CLI → API):

| Action | SDK Method | Params | Returns | Destructive |
|--------|-----------|--------|---------|-------------|
| `help` | — (built-in) | — | Catalog | no |
| `schema` | — (built-in) | `action: string` (required) | Schema | no |
| `server.health` | `health()` | — | `{ ok: true }` | no |
| `collections.list` | `collections_list()` | — | `CollectionDescription[]` | no |
| `collections.get` | `collection_get(name)` | `name: string` (required) | `CollectionInfo` (raw Value) | no |
| `collection.create` | `collection_create(name, body)` | `name: string`, `size: integer`, `distance: string` | Value | no |
| `collection.delete` | `collection_delete(name)` | `name: string` | void | **yes** |
| `point.upsert` | `point_upsert_batched(collection, points)` | `collection: string`, `points: array` | Value | no |
| `point.search` | `point_search(collection, req)` | `collection: string`, `vector: array`, `limit: integer`, `filter?`, `with_payload?`, `score_threshold?` | Value | no |
| `point.query` | `point_query(collection, body)` | `collection: string`, `query: object` | Value | no |
| `point.scroll` | `point_scroll(collection, body)` | `collection: string`, `scroll?: object` | Value | no |
| `point.count` | `point_count(collection, body)` | `collection: string`, `filter?: object` | Value | no |
| `point.delete` | `point_delete(collection, body)` | `collection: string`, `points?: array`, `filter?: object` | Value | **yes** |
| `snapshot.create` | `snapshot_create(collection)` | `collection: string` | SnapshotInfo | no |
| `index.create` | `index_create(collection, req)` | `collection: string`, `field_name: string`, `field_schema: object` | Value | no |

**Surface wiring:**
- MCP: `mcp/services/qdrant.rs` forwards to `dispatch::qdrant::dispatch()`
- CLI: `cli/qdrant.rs` calls `mcp::services::qdrant::dispatch()` directly (Tier-2 shim)
- API: `api/services/qdrant.rs` calls `dispatch::qdrant::dispatch_with_client()`

**Auth:** `QDRANT_URL` is required. `QDRANT_API_KEY` is optional — if present, sent as `api-key` header; if absent, `Auth::None` is used (unauthenticated local instance).

## Collections

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|------------|-----------------|------|-----|-----|-----|
| GET | /collections | `collections_list()` | `collections.list` | ✅ | ✅ | ✅ | ✅ |
| DELETE | /collections/{collection_name} | `collection_delete(name)` | `collection.delete` | ✅ | ✅ | ✅ | ✅ |
| GET | /collections/{collection_name} | `collection_get(name)` | `collections.get` | ✅ | ✅ | ✅ | ✅ |
| PATCH | /collections/{collection_name} | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /collections/{collection_name} | `collection_create(name, body)` | `collection.create` | ✅ | ✅ | ✅ | ✅ |
| GET | /collections/{collection_name}/exists | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /collections/{collection_name}/optimizations | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

## Points

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|------------|-----------------|------|-----|-----|-----|
| POST | /collections/{collection_name}/facet | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /collections/{collection_name}/points | `point_upsert_batched(col, points)` | `point.upsert` | ✅ | ✅ | ✅ | ✅ |
| POST | /collections/{collection_name}/points/batch | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/count | `point_count(col, body)` | `point.count` | ✅ | ✅ | ✅ | ✅ |
| POST | /collections/{collection_name}/points/delete | `point_delete(col, body)` | `point.delete` | ✅ | ✅ | ✅ | ✅ |
| POST | /collections/{collection_name}/points/payload | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /collections/{collection_name}/points/payload | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/payload/clear | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/payload/delete | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/scroll | `point_scroll(col, body)` | `point.scroll` | ✅ | ✅ | ✅ | ✅ |
| PUT | /collections/{collection_name}/points/vectors | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/vectors/delete | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /collections/{collection_name}/points/{id} | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

## Search

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|------------|-----------------|------|-----|-----|-----|
| POST | /collections/{collection_name}/points/discover | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/discover/batch | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/query | `point_query(col, body)` | `point.query` | ✅ | ✅ | ✅ | ✅ |
| POST | /collections/{collection_name}/points/query/batch | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/query/groups | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/recommend | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/recommend/batch | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/recommend/groups | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/search | `point_search(col, req)` | `point.search` | ✅ | ✅ | ✅ | ✅ |
| POST | /collections/{collection_name}/points/search/batch | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/search/groups | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/search/matrix/offsets | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/search/matrix/pairs | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

## Aliases

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|------------|-----------------|------|-----|-----|-----|
| GET | /aliases | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/aliases | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /collections/{collection_name}/aliases | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

## Indexes

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|------------|-----------------|------|-----|-----|-----|
| PUT | /collections/{collection_name}/index | `index_create(col, req)` | `index.create` | ✅ | ✅ | ✅ | ✅ |
| DELETE | /collections/{collection_name}/index/{field_name} | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

## Distributed

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|------------|-----------------|------|-----|-----|-----|
| GET | /cluster | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /cluster/peer/{peer_id} | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /cluster/recover | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /cluster/telemetry | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /collections/{collection_name}/cluster | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/cluster | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /collections/{collection_name}/shards | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /collections/{collection_name}/shards | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/shards/delete | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

## Snapshots

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|------------|-----------------|------|-----|-----|-----|
| GET | /collections/{collection_name}/shards/{shard_id}/snapshot | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /collections/{collection_name}/shards/{shard_id}/snapshots | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/shards/{shard_id}/snapshots | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /collections/{collection_name}/shards/{shard_id}/snapshots/recover | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/shards/{shard_id}/snapshots/upload | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /collections/{collection_name}/shards/{shard_id}/snapshots/{snapshot_name} | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /collections/{collection_name}/shards/{shard_id}/snapshots/{snapshot_name} | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /collections/{collection_name}/snapshots | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/snapshots | `snapshot_create(col)` | `snapshot.create` | ✅ | ✅ | ✅ | ✅ |
| PUT | /collections/{collection_name}/snapshots/recover | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/snapshots/upload | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /collections/{collection_name}/snapshots/{snapshot_name} | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /collections/{collection_name}/snapshots/{snapshot_name} | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /snapshots | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /snapshots | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /snapshots/{snapshot_name} | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /snapshots/{snapshot_name} | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

## Service

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|------------|-----------------|------|-----|-----|-----|
| GET | / | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /healthz | `health()` | `server.health` | ✅ | ✅ | ✅ | ✅ |
| GET | /livez | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /metrics | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /readyz | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /telemetry | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

## Beta

| Method | Endpoint | SDK Method | Dispatch Action | Impl | MCP | CLI | API |
|--------|----------|------------|-----------------|------|-----|-----|-----|
| DELETE | /issues | — | — | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /issues | — | — | ⬜ | ⬜ | ⬜ | ⬜ |

## Notes

- The CLI shim (`cli/qdrant.rs`) is Tier-2: it calls `mcp::services::qdrant::dispatch()` directly rather than the shared dispatch layer. This is correct current behavior but does not match the canonical Tier-1 typed pattern used by radarr.
- The SDK client (`lab-apis/src/qdrant/client.rs`) now implements 13 methods: `health()`, `collections_list()`, `collection_get()`, `collection_create()`, `collection_delete()`, `point_upsert()`, `point_upsert_batched()`, `point_search()`, `point_query()`, `point_scroll()`, `point_count()`, `point_delete()`, `snapshot_create()`, and `index_create()`.
- `point_upsert_batched()` auto-chunks large point sets at 500 points/chunk and fails fast (returning chunk context) if any chunk fails.
- `QDRANT_API_KEY` is optional. `QDRANT_URL` is required.
