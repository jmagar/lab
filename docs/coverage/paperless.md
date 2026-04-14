# Paperless-ngx API Coverage

**Last updated:** 2026-04-13 (bead lab-hq03.13 — 10 new actions)
**Source spec:** docs/upstream-api/paperless.md
**Auth:** `Authorization: Token <token>` (API token from user account settings)

## Summary

Full dispatch layer implemented. All CRUD operations for documents, tags, correspondents,
and document types are wired. Statistics and tasks endpoints are also available. 10 new
actions added: document upload (multipart), document bulk-edit, document download (base64),
tag PATCH, saved-view list/create, custom-field list/create, and storage-path list/create.

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

## Config / Auth

| Env Var | Required | Description |
|---------|----------|-------------|
| `PAPERLESS_URL` | yes | Base URL (e.g. `https://paperless.example.com`) |
| `PAPERLESS_TOKEN` | yes | API token (`Authorization: Token <value>`) |
| `PAPERLESS_API_KEY` | no | Alias accepted for `PAPERLESS_TOKEN` |

## CLI Usage

The CLI is a flat action+params shim (`PaperlessArgs { action, params }`). There are no typed subcommands.

```bash
lab paperless help
lab paperless statistics
lab paperless documents.list --params '{"query":"invoice","page":1}'
lab paperless documents.get --params '{"id":42}'
lab paperless documents.metadata --params '{"id":42}'
lab paperless documents.update --params '{"id":42,"title":"New Title"}'
lab paperless documents.delete --params '{"id":42}'
lab paperless tags.list
lab paperless tags.get --params '{"id":3}'
lab paperless tags.create --params '{"name":"work"}'
lab paperless tags.delete --params '{"id":3}'
lab paperless correspondents.list
lab paperless correspondents.get --params '{"id":1}'
lab paperless correspondents.create --params '{"name":"ACME Corp"}'
lab paperless correspondents.delete --params '{"id":1}'
lab paperless document_types.list
lab paperless document_types.get --params '{"id":2}'
lab paperless document_types.create --params '{"name":"Invoice"}'
lab paperless document_types.delete --params '{"id":2}'
lab paperless tasks.list
```

## Action Coverage Matrix

| Action | Endpoint | MCP | CLI | HTTP API | Live-tested |
|--------|----------|-----|-----|----------|-------------|
| `documents.list` | `GET /api/documents/` | yes | yes | yes | ✅ |
| `documents.get` | `GET /api/documents/{id}/` | yes | yes | yes | no |
| `documents.metadata` | `GET /api/documents/{id}/metadata/` | yes | yes | yes | no |
| `documents.update` | `PATCH /api/documents/{id}/` | yes | yes | yes | no |
| `documents.delete` | `DELETE /api/documents/{id}/` | yes | yes | yes | no |
| `document.upload` | `POST /api/documents/post_document/` (multipart) | yes | yes | yes | no |
| `document.bulk-edit` | `POST /api/documents/bulk_edit/` | yes | yes | yes | no |
| `document.download` | `GET /api/documents/{id}/download/` | yes | yes | yes | no |
| `tags.list` | `GET /api/tags/` | yes | yes | yes | ✅ |
| `tags.get` | `GET /api/tags/{id}/` | yes | yes | yes | no |
| `tags.create` | `POST /api/tags/` | yes | yes | yes | no |
| `tags.delete` | `DELETE /api/tags/{id}/` | yes | yes | yes | no |
| `tag.update` | `PATCH /api/tags/{id}/` | yes | yes | yes | no |
| `correspondents.list` | `GET /api/correspondents/` | yes | yes | yes | no |
| `correspondents.get` | `GET /api/correspondents/{id}/` | yes | yes | yes | no |
| `correspondents.create` | `POST /api/correspondents/` | yes | yes | yes | no |
| `correspondents.delete` | `DELETE /api/correspondents/{id}/` | yes | yes | yes | no |
| `document_types.list` | `GET /api/document_types/` | yes | yes | yes | no |
| `document_types.get` | `GET /api/document_types/{id}/` | yes | yes | yes | no |
| `document_types.create` | `POST /api/document_types/` | yes | yes | yes | no |
| `document_types.delete` | `DELETE /api/document_types/{id}/` | yes | yes | yes | no |
| `saved-view.list` | `GET /api/saved_views/` | yes | yes | yes | no |
| `saved-view.create` | `POST /api/saved_views/` | yes | yes | yes | no |
| `custom-field.list` | `GET /api/custom_fields/` | yes | yes | yes | no |
| `custom-field.create` | `POST /api/custom_fields/` | yes | yes | yes | no |
| `storage-path.list` | `GET /api/storage_paths/` | yes | yes | yes | no |
| `storage-path.create` | `POST /api/storage_paths/` | yes | yes | yes | no |
| `statistics` | `GET /api/statistics/` | yes | yes | yes | ✅ |
| `tasks.list` | `GET /api/tasks/` | yes | yes | yes | no |

## Action Details

### `documents.list`
- **Params:**
  - `query` (string, optional) — full-text search query
  - `page` (integer, optional) — page number (1-based)
  - `page_size` (integer, optional) — results per page
  - `ordering` (string, optional) — sort field; prefix with `-` for descending (e.g. `-created`)
  - `correspondent` (integer, optional) — filter by correspondent ID
  - `document_type` (integer, optional) — filter by document type ID
  - `tags__id__all` (string, optional) — comma-separated tag IDs that must all be present
- **Returns:** paginated document list (raw JSON from Paperless-ngx)

### `documents.get`
- **Params:** `id` (integer, required)
- **Returns:** single document record (raw JSON)

### `documents.metadata`
- **Params:** `id` (integer, required)
- **Returns:** full document metadata (raw JSON from `/api/documents/{id}/metadata/`)

### `documents.update` *(destructive)*
- **Params:**
  - `id` (integer, required) — document ID
  - `title` (string, optional)
  - `created` (string, optional) — ISO-8601 date
  - `correspondent` (integer, optional) — correspondent ID
  - `document_type` (integer, optional) — document type ID
  - `tags` (json array, optional) — array of tag IDs
  - `payload` (json, optional) — alternative: full JSON body (overrides individual params when present)
- **Returns:** updated document record (raw JSON)
- **Notes:** Uses HTTP PATCH. Requires `"confirm": true` in HTTP API params; elicitation in MCP; `-y`/`--yes` in CLI.

### `documents.delete` *(destructive)*
- **Params:** `id` (integer, required)
- **Returns:** `null`
- **Notes:** Permanently deletes the document and all its files.

### `tags.list`
- **Params:** none
- **Returns:** tag list (raw JSON)

### `tags.get`
- **Params:** `id` (integer, required)
- **Returns:** single tag record (raw JSON)

### `tags.create` *(destructive)*
- **Params:**
  - `name` (string, required)
  - `colour` (string, optional) — hex color string (e.g. `#ff0000`); serialized as `color` in the API body
  - `is_inbox_tag` (bool, optional) — whether this is the special inbox tag
  - `match_expr` (string, optional) — matching expression for auto-tagging rules (serialized as `match`)
  - `matching_algorithm` (integer, optional) — matching algorithm ID
  - `payload` (json, optional) — alternative: full JSON body (overrides individual params when present)
- **Returns:** created tag record (raw JSON)

### `tags.delete` *(destructive)*
- **Params:** `id` (integer, required)
- **Returns:** `null`

### `correspondents.list`
- **Params:** none
- **Returns:** correspondent list (raw JSON)

### `correspondents.get`
- **Params:** `id` (integer, required)
- **Returns:** single correspondent record (raw JSON)

### `correspondents.create` *(destructive)*
- **Params:**
  - `name` (string, required)
  - `match_expr` (string, optional) — matching expression for auto-assign rules (serialized as `match`)
  - `matching_algorithm` (integer, optional) — matching algorithm ID
  - `payload` (json, optional) — alternative: full JSON body (overrides individual params when present)
- **Returns:** created correspondent record (raw JSON)

### `correspondents.delete` *(destructive)*
- **Params:** `id` (integer, required)
- **Returns:** `null`

### `document_types.list`
- **Params:** none
- **Returns:** document type list (raw JSON)

### `document_types.get`
- **Params:** `id` (integer, required)
- **Returns:** single document type record (raw JSON)

### `document_types.create` *(destructive)*
- **Params:**
  - `name` (string, required)
  - `match_expr` (string, optional) — matching expression for auto-classify rules (serialized as `match`)
  - `matching_algorithm` (integer, optional) — matching algorithm ID
  - `payload` (json, optional) — alternative: full JSON body (overrides individual params when present)
- **Returns:** created document type record (raw JSON)

### `document_types.delete` *(destructive)*
- **Params:** `id` (integer, required)
- **Returns:** `null`

### `statistics`
- **Params:** none
- **Returns:** instance statistics (raw JSON) — includes `documents_total`, inbox count, etc.
- **Notes:** Also used as the health probe (`GET /api/statistics/` is the lightest authenticated endpoint).

### `tasks.list`
- **Params:** none
- **Returns:** async task list (raw JSON from `/api/tasks/`)

### `document.upload`
- **Params:**
  - `file_base64` (string, required) — base64-encoded file content
  - `filename` (string, required) — file name (e.g. `invoice.pdf`)
  - `title` (string, optional) — document title
  - `correspondent` (integer, optional) — correspondent ID
  - `document_type` (integer, optional) — document type ID
  - `tags` (json array, optional) — array of tag IDs
- **Returns:** task ID JSON from Paperless-ngx (`{"task_id": "..."}`)
- **Notes:** Sends `multipart/form-data` to `/api/documents/post_document/`.

### `document.bulk-edit` *(destructive)*
- **Params:**
  - `documents` (json array, required) — array of document IDs
  - `method` (string, required) — bulk operation (`delete`, `set_correspondent`, `set_document_type`, `add_tag`, `remove_tag`)
  - `parameters` (json, optional) — method-specific parameters object
- **Returns:** result JSON from Paperless-ngx
- **Notes:** Marked destructive because `method: "delete"` permanently removes documents.

### `document.download`
- **Params:**
  - `id` (integer, required) — document ID
  - `original` (bool, optional) — if true, returns the original file before OCR post-processing
- **Returns:** `{"content_base64": "...", "size": N, "content_type": "application/octet-stream"}`
- **Notes:** Downloads the document file and returns it base64-encoded for JSON transport.

### `tag.update`
- **Params:**
  - `id` (integer, required) — tag ID
  - `name` (string, optional) — tag name
  - `colour` (string, optional) — hex colour string (serialized as `color` in the API body)
  - `match` (string, optional) — matching expression
- **Returns:** updated tag record (raw JSON)
- **Notes:** Uses HTTP PATCH `/api/tags/{id}/`.

### `saved-view.list`
- **Params:** none
- **Returns:** saved view list (raw JSON from `/api/saved_views/`)

### `saved-view.create`
- **Params:**
  - `payload` (json, required) — saved view definition (`name`, `filter_rules`, `sort_field`, etc.)
- **Returns:** created saved view record (raw JSON)

### `custom-field.list`
- **Params:** none
- **Returns:** custom field list (raw JSON from `/api/custom_fields/`)

### `custom-field.create`
- **Params:**
  - `name` (string, required) — field name
  - `data_type` (string, required) — data type (`string`, `integer`, `date`, `boolean`, `url`, `monetary`)
- **Returns:** created custom field record (raw JSON)

### `storage-path.list`
- **Params:** none
- **Returns:** storage path list (raw JSON from `/api/storage_paths/`)

### `storage-path.create`
- **Params:**
  - `payload` (json, required) — storage path definition (`name`, `path`, etc.)
- **Returns:** created storage path record (raw JSON)

## Destructive Actions

The following actions require `"confirm": true` in the HTTP API params, elicitation in MCP,
and `-y` / `--yes` in CLI:

- `documents.update` — PATCH modifies document metadata
- `documents.delete` — permanently deletes a document
- `document.bulk-edit` — can permanently delete documents or bulk-modify metadata
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

- Correspondent partial update (`PATCH /api/correspondents/{id}/`) — deferred
- Document type partial update (`PATCH /api/document_types/{id}/`) — deferred
- Saved view update/delete (`PATCH/DELETE /api/saved_views/{id}/`) — deferred
- Storage path update/delete (`PATCH/DELETE /api/storage_paths/{id}/`) — deferred
- Users / groups management — deferred
