# Paperless-ngx API Coverage

**Last updated:** 2026-04-12
**Source spec:** docs/upstream-api/paperless.md
**Auth:** `Authorization: Token <token>` (API token from user account settings)

## Summary

Full dispatch layer implemented. All CRUD operations for documents, tags, correspondents,
and document types are wired. Statistics and tasks endpoints are also available.

## Live Test Evidence

Live smoke tests run 2026-04-12 against `https://paperless.tootie.tv`.

> **Note:** The env var `PAPERLESS_API_KEY` is also accepted as an alias for `PAPERLESS_TOKEN`.

| Surface | Command | Result |
|---------|---------|--------|
| CLI | `lab paperless statistics` | 6 documents, 4 tags |
| CLI | `lab paperless documents.list` | 6 documents |
| CLI | `lab paperless tags.list` | 4 tags |
| MCP | `mcporter call lab.paperless action=statistics` | `ok=true`, documents_total=6 |
| MCP | `mcporter call lab.paperless action=documents.list` | `ok=true`, count=6 |
| API | `POST /v1/paperless {"action":"statistics"}` | documents_total=6, tag_count=4 |
| API | `POST /v1/paperless {"action":"documents.list"}` | count=6 |

## Action Coverage Matrix

| Action | Endpoint | MCP | CLI | HTTP API | Live-tested |
|--------|----------|-----|-----|----------|-------------|
| `documents.list` | `GET /api/documents/` | yes | passthrough | yes | ✅ |
| `documents.get` | `GET /api/documents/{id}/` | yes | passthrough | yes | no |
| `documents.metadata` | `GET /api/documents/{id}/metadata/` | yes | passthrough | yes | no |
| `documents.update` | `PATCH /api/documents/{id}/` | yes | passthrough | yes | no |
| `documents.delete` | `DELETE /api/documents/{id}/` | yes | passthrough | yes | no |
| `tags.list` | `GET /api/tags/` | yes | passthrough | yes | ✅ |
| `tags.get` | `GET /api/tags/{id}/` | yes | passthrough | yes | no |
| `tags.create` | `POST /api/tags/` | yes | passthrough | yes | no |
| `tags.delete` | `DELETE /api/tags/{id}/` | yes | passthrough | yes | no |
| `correspondents.list` | `GET /api/correspondents/` | yes | passthrough | yes | no |
| `correspondents.get` | `GET /api/correspondents/{id}/` | yes | passthrough | yes | no |
| `correspondents.create` | `POST /api/correspondents/` | yes | passthrough | yes | no |
| `correspondents.delete` | `DELETE /api/correspondents/{id}/` | yes | passthrough | yes | no |
| `document_types.list` | `GET /api/document_types/` | yes | passthrough | yes | no |
| `document_types.get` | `GET /api/document_types/{id}/` | yes | passthrough | yes | no |
| `document_types.create` | `POST /api/document_types/` | yes | passthrough | yes | no |
| `document_types.delete` | `DELETE /api/document_types/{id}/` | yes | passthrough | yes | no |
| `statistics` | `GET /api/statistics/` | yes | passthrough | yes | ✅ |
| `tasks.list` | `GET /api/tasks/` | yes | passthrough | yes | no |

## Destructive Actions

The following actions require `"confirm": true` in the HTTP API, elicitation in MCP,
and `-y` / `--yes` in CLI:

- `documents.update` — PATCH modifies document metadata
- `documents.delete` — permanently deletes a document
- `tags.create` — creates a persistent tag
- `tags.delete` — removes a tag from all documents
- `correspondents.create` — creates a persistent correspondent
- `correspondents.delete` — removes a correspondent from all documents
- `document_types.create` — creates a persistent document type
- `document_types.delete` — removes a document type from all documents

## Health Check

Probe: `GET /api/statistics/` — lightest authenticated endpoint that returns meaningful
data. Used by `lab doctor` and the `ServiceClient::health()` implementation.

## Not Yet Implemented

- Document upload (`POST /api/documents/post_document/`) — multipart form, deferred
- Tag partial update (`PATCH /api/tags/{id}/`) — deferred
- Correspondent partial update (`PATCH /api/correspondents/{id}/`) — deferred
- Document type partial update (`PATCH /api/document_types/{id}/`) — deferred
- Saved views (`GET/POST/PATCH/DELETE /api/saved_views/`) — deferred
- Storage paths (`GET /api/storage_paths/`) — deferred
- Users / groups management — deferred
