# Qdrant API Coverage

**Last updated:** 2026-04-12
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

## Collections

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /collections | get_collections | ✅ | ✅ | ✅ | ✅ |
| DELETE | /collections/{collection_name} | delete_collection | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /collections/{collection_name} | get_collection | ✅ | ✅ | ✅ | ✅ |
| PATCH | /collections/{collection_name} | update_collection | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /collections/{collection_name} | create_collection | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /collections/{collection_name}/exists | collection_exists | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /collections/{collection_name}/optimizations | get_optimizations | ⬜ | ⬜ | ⬜ | ⬜ |

## Points

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| POST | /collections/{collection_name}/facet | facet | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points | get_points | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /collections/{collection_name}/points | upsert_points | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/batch | batch_update | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/count | count_points | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/delete | delete_points | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/payload | set_payload | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /collections/{collection_name}/points/payload | overwrite_payload | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/payload/clear | clear_payload | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/payload/delete | delete_payload | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/scroll | scroll_points | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /collections/{collection_name}/points/vectors | update_vectors | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/vectors/delete | delete_vectors | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /collections/{collection_name}/points/{id} | get_point | ⬜ | ⬜ | ⬜ | ⬜ |

## Search

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| POST | /collections/{collection_name}/points/discover | discover_points | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/discover/batch | discover_batch_points | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/query | query_points | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/query/batch | query_batch_points | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/query/groups | query_points_groups | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/recommend | recommend_points | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/recommend/batch | recommend_batch_points | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/recommend/groups | recommend_point_groups | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/search | search_points | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/search/batch | search_batch_points | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/search/groups | search_point_groups | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/search/matrix/offsets | search_matrix_offsets | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/points/search/matrix/pairs | search_matrix_pairs | ⬜ | ⬜ | ⬜ | ⬜ |

## Aliases

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /aliases | get_collections_aliases | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/aliases | update_aliases | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /collections/{collection_name}/aliases | get_collection_aliases | ⬜ | ⬜ | ⬜ | ⬜ |

## Indexes

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| PUT | /collections/{collection_name}/index | create_field_index | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /collections/{collection_name}/index/{field_name} | delete_field_index | ⬜ | ⬜ | ⬜ | ⬜ |

## Distributed

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /cluster | cluster_status | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /cluster/peer/{peer_id} | remove_peer | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /cluster/recover | recover_current_peer | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /cluster/telemetry | cluster_telemetry | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /collections/{collection_name}/cluster | collection_cluster_info | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/cluster | update_collection_cluster | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /collections/{collection_name}/shards | list_shard_keys | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /collections/{collection_name}/shards | create_shard_key | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/shards/delete | delete_shard_key | ⬜ | ⬜ | ⬜ | ⬜ |

## Snapshots

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /collections/{collection_name}/shards/{shard_id}/snapshot | stream_shard_snapshot | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /collections/{collection_name}/shards/{shard_id}/snapshots | list_shard_snapshots | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/shards/{shard_id}/snapshots | create_shard_snapshot | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /collections/{collection_name}/shards/{shard_id}/snapshots/recover | recover_shard_from_snapshot | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/shards/{shard_id}/snapshots/upload | recover_shard_from_uploaded_snapshot | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /collections/{collection_name}/shards/{shard_id}/snapshots/{snapshot_name} | delete_shard_snapshot | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /collections/{collection_name}/shards/{shard_id}/snapshots/{snapshot_name} | get_shard_snapshot | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /collections/{collection_name}/snapshots | list_snapshots | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/snapshots | create_snapshot | ⬜ | ⬜ | ⬜ | ⬜ |
| PUT | /collections/{collection_name}/snapshots/recover | recover_from_snapshot | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /collections/{collection_name}/snapshots/upload | recover_from_uploaded_snapshot | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /collections/{collection_name}/snapshots/{snapshot_name} | delete_snapshot | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /collections/{collection_name}/snapshots/{snapshot_name} | get_snapshot | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /snapshots | list_full_snapshots | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /snapshots | create_full_snapshot | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /snapshots/{snapshot_name} | delete_full_snapshot | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /snapshots/{snapshot_name} | get_full_snapshot | ⬜ | ⬜ | ⬜ | ⬜ |

## Service

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | / | root | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /healthz | healthz | ✅ | ✅ | ✅ | ✅ |
| GET | /livez | livez | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /metrics | metrics | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /readyz | readyz | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /telemetry | telemetry | ⬜ | ⬜ | ⬜ | ⬜ |

## Beta

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| DELETE | /issues | clear_issues | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /issues | get_issues | ⬜ | ⬜ | ⬜ | ⬜ |
