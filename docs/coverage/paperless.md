# Paperless-ngx API Coverage

**Last updated:** 2026-04-14
**Source spec:** docs/upstream-api/paperless.md
**Auth:** `Authorization: Token <token>` (API token from user account settings)

## Summary

Full dispatch layer implemented across SDK (lab-apis), shared dispatch (crates/lab/src/dispatch),
and all three product surfaces (CLI, MCP, HTTP API). All CRUD operations for documents, tags,
correspondents, and document types are wired. Statistics, tasks, and extended operations
(upload, bulk-edit, download, tag update, saved views, custom fields, storage paths) fully
implemented.

## Implementation Status

**SDK (lab-apis):** Fully implemented with `PaperlessClient` methods covering all actions.
**Dispatch (crates/lab/src/dispatch/paperless):** Complete with catalog, params, dispatch routing.
**CLI (crates/lab/src/cli/paperless.rs):** Thin dispatch shim using action+params pattern.
**MCP (crates/lab/src/registry.rs):** Test-verified; delegates to dispatch.
**API (crates/lab/src/api/services/paperless.rs):** Full route handler with shared dispatch integration.

All surfaces route through the shared dispatch layer (`crate::dispatch::paperless`).
Destructive actions require confirmation per standard protocol (MCP elicitation, CLI `-y`/`--yes`, API `"confirm": true` in params).

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

All actions listed below are fully wired across all three surfaces (MCP, CLI, HTTP API) via the shared dispatch layer. Built-in actions `help` and `schema` are available on every surface.

| Action | Endpoint | Destructive | MCP | CLI | HTTP API |
|--------|----------|-------------|-----|-----|----------|
| `documents.list` | `GET /api/documents/` | no | yes | yes | yes |
| `documents.get` | `GET /api/documents/{id}/` | no | yes | yes | yes |
| `documents.metadata` | `GET /api/documents/{id}/metadata/` | no | yes | yes | yes |
| `documents.update` | `PATCH /api/documents/{id}/` | yes | yes | yes | yes |
| `documents.delete` | `DELETE /api/documents/{id}/` | yes | yes | yes | yes |
| `document.upload` | `POST /api/documents/post_document/` (multipart) | no | yes | yes | yes |
| `document.bulk-edit` | `POST /api/documents/bulk_edit/` | yes | yes | yes | yes |
| `document.download` | `GET /api/documents/{id}/download/` | no | yes | yes | yes |
| `tags.list` | `GET /api/tags/` | no | yes | yes | yes |
| `tags.get` | `GET /api/tags/{id}/` | no | yes | yes | yes |
| `tags.create` | `POST /api/tags/` | yes | yes | yes | yes |
| `tags.delete` | `DELETE /api/tags/{id}/` | yes | yes | yes | yes |
| `tag.update` | `PATCH /api/tags/{id}/` | no | yes | yes | yes |
| `correspondents.list` | `GET /api/correspondents/` | no | yes | yes | yes |
| `correspondents.get` | `GET /api/correspondents/{id}/` | no | yes | yes | yes |
| `correspondents.create` | `POST /api/correspondents/` | yes | yes | yes | yes |
| `correspondents.delete` | `DELETE /api/correspondents/{id}/` | yes | yes | yes | yes |
| `document_types.list` | `GET /api/document_types/` | no | yes | yes | yes |
| `document_types.get` | `GET /api/document_types/{id}/` | no | yes | yes | yes |
| `document_types.create` | `POST /api/document_types/` | yes | yes | yes | yes |
| `document_types.delete` | `DELETE /api/document_types/{id}/` | yes | yes | yes | yes |
| `saved-view.list` | `GET /api/saved_views/` | no | yes | yes | yes |
| `saved-view.create` | `POST /api/saved_views/` | no | yes | yes | yes |
| `custom-field.list` | `GET /api/custom_fields/` | no | yes | yes | yes |
| `custom-field.create` | `POST /api/custom_fields/` | no | yes | yes | yes |
| `storage-path.list` | `GET /api/storage_paths/` | no | yes | yes | yes |
| `storage-path.create` | `POST /api/storage_paths/` | no | yes | yes | yes |
| `statistics` | `GET /api/statistics/` | no | yes | yes | yes |
| `tasks.list` | `GET /api/tasks/` | no | yes | yes | yes |

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
- **Returns:** paginated document list (JSON object with results, count, etc.)
- **Wiring:** SDK `documents_list()` → dispatch `documents.list` → CLI/MCP/API

### `documents.get`
- **Params:** `id` (integer, required)
- **Returns:** single document record (JSON object)
- **Wiring:** SDK `document_get()` → dispatch `documents.get` → CLI/MCP/API

### `documents.metadata`
- **Params:** `id` (integer, required)
- **Returns:** full document metadata (JSON object from `/api/documents/{id}/metadata/`)
- **Wiring:** SDK `document_metadata()` → dispatch `documents.metadata` → CLI/MCP/API

### `documents.update` *(destructive)*
- **Params:**
  - `id` (integer, required) — document ID
  - `title` (string, optional)
  - `created` (string, optional) — ISO-8601 date
  - `correspondent` (integer, optional) — correspondent ID
  - `document_type` (integer, optional) — document type ID
  - `tags` (json array, optional) — array of tag IDs
  - `payload` (json, optional) — alternative: full JSON body (overrides individual params when present)
- **Returns:** updated document record (JSON object)
- **Notes:** HTTP PATCH via SDK. Requires confirmation per standard protocol.
- **Wiring:** SDK `document_update()` → dispatch `documents.update` → CLI/MCP/API

### `documents.delete` *(destructive)*
- **Params:** `id` (integer, required)
- **Returns:** `null`
- **Notes:** Permanently deletes the document and all its files. Requires confirmation per standard protocol.
- **Wiring:** SDK `document_delete()` → dispatch `documents.delete` → CLI/MCP/API

### `tags.list`
- **Params:** none
- **Returns:** tag list (JSON object with results)
- **Wiring:** SDK `tags_list()` → dispatch `tags.list` → CLI/MCP/API

### `tags.get`
- **Params:** `id` (integer, required)
- **Returns:** single tag record (JSON object)
- **Wiring:** SDK `tag_get()` → dispatch `tags.get` → CLI/MCP/API

### `tags.create` *(destructive)*
- **Params:**
  - `name` (string, required)
  - `colour` (string, optional) — hex color string (e.g. `#ff0000`); serialized as `color` in the API body
  - `is_inbox_tag` (bool, optional) — whether this is the special inbox tag
  - `match_expr` (string, optional) — matching expression for auto-tagging rules (serialized as `match`)
  - `matching_algorithm` (integer, optional) — matching algorithm ID
  - `payload` (json, optional) — alternative: full JSON body (overrides individual params when present)
- **Returns:** created tag record (JSON object)
- **Notes:** Creates a persistent tag. Requires confirmation per standard protocol.
- **Wiring:** SDK `tag_create()` → dispatch `tags.create` → CLI/MCP/API

### `tags.delete` *(destructive)*
- **Params:** `id` (integer, required)
- **Returns:** `null`
- **Notes:** Removes a tag from all documents. Requires confirmation per standard protocol.
- **Wiring:** SDK `tag_delete()` → dispatch `tags.delete` → CLI/MCP/API

### `correspondents.list`
- **Params:** none
- **Returns:** correspondent list (JSON object with results)
- **Wiring:** SDK `correspondents_list()` → dispatch `correspondents.list` → CLI/MCP/API

### `correspondents.get`
- **Params:** `id` (integer, required)
- **Returns:** single correspondent record (JSON object)
- **Wiring:** SDK `correspondent_get()` → dispatch `correspondents.get` → CLI/MCP/API

### `correspondents.create` *(destructive)*
- **Params:**
  - `name` (string, required)
  - `match_expr` (string, optional) — matching expression for auto-assign rules (serialized as `match`)
  - `matching_algorithm` (integer, optional) — matching algorithm ID
  - `payload` (json, optional) — alternative: full JSON body (overrides individual params when present)
- **Returns:** created correspondent record (JSON object)
- **Notes:** Creates a persistent correspondent. Requires confirmation per standard protocol.
- **Wiring:** SDK `correspondent_create()` → dispatch `correspondents.create` → CLI/MCP/API

### `correspondents.delete` *(destructive)*
- **Params:** `id` (integer, required)
- **Returns:** `null`
- **Notes:** Removes a correspondent from all documents. Requires confirmation per standard protocol.
- **Wiring:** SDK `correspondent_delete()` → dispatch `correspondents.delete` → CLI/MCP/API

### `document_types.list`
- **Params:** none
- **Returns:** document type list (JSON object with results)
- **Wiring:** SDK `document_types_list()` → dispatch `document_types.list` → CLI/MCP/API

### `document_types.get`
- **Params:** `id` (integer, required)
- **Returns:** single document type record (JSON object)
- **Wiring:** SDK `document_type_get()` → dispatch `document_types.get` → CLI/MCP/API

### `document_types.create` *(destructive)*
- **Params:**
  - `name` (string, required)
  - `match_expr` (string, optional) — matching expression for auto-classify rules (serialized as `match`)
  - `matching_algorithm` (integer, optional) — matching algorithm ID
  - `payload` (json, optional) — alternative: full JSON body (overrides individual params when present)
- **Returns:** created document type record (JSON object)
- **Notes:** Creates a persistent document type. Requires confirmation per standard protocol.
- **Wiring:** SDK `document_type_create()` → dispatch `document_types.create` → CLI/MCP/API

### `document_types.delete` *(destructive)*
- **Params:** `id` (integer, required)
- **Returns:** `null`
- **Notes:** Removes a document type from all documents. Requires confirmation per standard protocol.
- **Wiring:** SDK `document_type_delete()` → dispatch `document_types.delete` → CLI/MCP/API

### `statistics`
- **Params:** none
- **Returns:** instance statistics (JSON object) — includes `documents_total`, inbox count, etc.
- **Notes:** Also used as the health probe via `ServiceClient::health()`. Lightest authenticated endpoint.
- **Wiring:** SDK `statistics()` → dispatch `statistics` → CLI/MCP/API

### `tasks.list`
- **Params:** none
- **Returns:** async task list (JSON object with results)
- **Wiring:** SDK `tasks_list()` → dispatch `tasks.list` → CLI/MCP/API

### `document.upload`
- **Params:**
  - `file_base64` (string, required) — base64-encoded file content
  - `filename` (string, required) — file name (e.g. `invoice.pdf`)
  - `title` (string, optional) — document title
  - `correspondent` (integer, optional) — correspondent ID
  - `document_type` (integer, optional) — document type ID
  - `tags` (json array, optional) — array of tag IDs
- **Returns:** task ID (JSON object `{"task_id": "..."}`)
- **Notes:** Sends `multipart/form-data` internally via SDK. File is decoded from base64.
- **Wiring:** SDK `document_upload()` → dispatch `document.upload` → CLI/MCP/API

### `document.bulk-edit` *(destructive)*
- **Params:**
  - `documents` (json array, required) — array of document IDs
  - `method` (string, required) — bulk operation (`delete`, `set_correspondent`, `set_document_type`, `add_tag`, `remove_tag`)
  - `parameters` (json, optional) — method-specific parameters object
- **Returns:** result JSON from Paperless-ngx
- **Notes:** Marked destructive because `method: "delete"` permanently removes documents. Requires confirmation per standard protocol.
- **Wiring:** SDK `document_bulk_edit()` → dispatch `document.bulk-edit` → CLI/MCP/API

### `document.download`
- **Params:**
  - `id` (integer, required) — document ID
  - `original` (bool, optional) — if true, returns the original file before OCR post-processing
- **Returns:** `{"content_base64": "...", "size": N, "content_type": "application/octet-stream"}`
- **Notes:** Downloads the document file and returns it base64-encoded for JSON transport.
- **Wiring:** SDK `document_download()` → dispatch `document.download` → CLI/MCP/API

### `tag.update`
- **Params:**
  - `id` (integer, required) — tag ID
  - `name` (string, optional) — tag name
  - `colour` (string, optional) — hex colour string (serialized as `color` in the API body)
  - `match` (string, optional) — matching expression
- **Returns:** updated tag record (JSON object)
- **Notes:** HTTP PATCH operation. Read-only per API (no confirmation required).
- **Wiring:** SDK `tag_update()` → dispatch `tag.update` → CLI/MCP/API

### `saved-view.list`
- **Params:** none
- **Returns:** saved view list (JSON object with results)
- **Wiring:** SDK `saved_views_list()` → dispatch `saved-view.list` → CLI/MCP/API

### `saved-view.create`
- **Params:**
  - `payload` (json, required) — saved view definition (`name`, `filter_rules`, `sort_field`, etc.)
- **Returns:** created saved view record (JSON object)
- **Wiring:** SDK `saved_view_create()` → dispatch `saved-view.create` → CLI/MCP/API

### `custom-field.list`
- **Params:** none
- **Returns:** custom field list (JSON object with results)
- **Wiring:** SDK `custom_fields_list()` → dispatch `custom-field.list` → CLI/MCP/API

### `custom-field.create`
- **Params:**
  - `name` (string, required) — field name
  - `data_type` (string, required) — data type (`string`, `integer`, `date`, `boolean`, `url`, `monetary`)
- **Returns:** created custom field record (JSON object)
- **Wiring:** SDK `custom_field_create()` → dispatch `custom-field.create` → CLI/MCP/API

### `storage-path.list`
- **Params:** none
- **Returns:** storage path list (JSON object with results)
- **Wiring:** SDK `storage_paths_list()` → dispatch `storage-path.list` → CLI/MCP/API

### `storage-path.create`
- **Params:**
  - `payload` (json, required) — storage path definition (`name`, `path`, etc.)
- **Returns:** created storage path record (JSON object)
- **Wiring:** SDK `storage_path_create()` → dispatch `storage-path.create` → CLI/MCP/API

## Destructive Actions

The following actions are marked `destructive: true` in the catalog and follow the standard confirmation protocol:

- `documents.update` — PATCH modifies document metadata
- `documents.delete` — permanently deletes a document
- `document.bulk-edit` — can permanently delete documents or bulk-modify metadata
- `tags.create` — creates a persistent tag
- `tags.delete` — removes a tag from all documents
- `correspondents.create` — creates a persistent correspondent
- `correspondents.delete` — removes a correspondent from all documents
- `document_types.create` — creates a persistent document type
- `document_types.delete` — removes a document type from all documents

Confirmation is enforced uniformly across all surfaces:
- **HTTP API:** Requires `"confirm": true` (boolean) in the params JSON object. Returns 400 with `kind: "confirmation_required"` if missing.
- **MCP:** Invokes elicitation (client confirmation dialog). Agents may bypass via `"confirm": true` in params.
- **CLI:** Requires `-y` / `--yes` flag. Without it on a TTY, prompts the user interactively. Without it on a non-TTY, refuses with an error.

## Health Check

Probe: `GET /api/statistics/` — lightest authenticated endpoint that returns meaningful
data. Used by `lab doctor` and the `ServiceClient::health()` implementation.

Implementation: `PaperlessClient::probe()` in SDK, integrated into `lab doctor`.

## Surface Implementation Details

### SDK (crates/lab-apis/src/paperless)
- `PaperlessClient` with 24 async methods covering all documented actions
- `Auth::Token` with `"Authorization: Token <key>"` header
- Types: `DocumentUpdateRequest`, `TagCreateRequest`, `TagUpdateRequest`, `DocumentBulkEditRequest`, `DocumentDownloadInfo`, etc.
- Multipart upload via `reqwest::multipart::Form` (internal to SDK)
- Base64 download wrapping (internal to SDK)
- Error type: `PaperlessError` wrapping `ApiError`

### Dispatch Layer (crates/lab/src/dispatch/paperless)
- `catalog.rs` — single source of truth for all action specs (31 actions including `help`/`schema`)
- `client.rs` — `require_client()` returns `Result<PaperlessClient, ToolError>` from env vars
- `params.rs` — param extraction and request-body construction helpers
- `dispatch.rs` — top-level action routing + `dispatch_with_client()` for pre-built clients
- Error conversion via `From<PaperlessError> for ToolError`

### CLI (crates/lab/src/cli/paperless.rs)
- Thin shim using `run_action_command()` helper
- Parses `action` (positional) and `--params` (JSON string)
- Delegates to shared dispatch layer
- Supports all destructive flag patterns (`-y`, `--yes`, `--no-confirm`, `--dry-run`)
- Output formatting via `OutputFormat` (Human table or JSON)

### MCP (crates/lab/src/registry.rs)
- Test module verifying catalog completeness
- Delegates to shared dispatch; tool registered in `registry.rs`
- Supports `help` and `schema` built-in actions
- Elicitation for destructive ops via standard MCP protocol

### HTTP API (crates/lab/src/api/services/paperless.rs)
- Route group at `POST /v1/paperless`
- Accepts `ActionRequest { action, params, confirm }`
- Uses shared `handle_action()` helper for destructive confirmation, param stripping, logging
- Delegates to dispatch; supports `help`/`schema` via full dispatch, others via `dispatch_with_client()` with pre-built client from `AppState`
- Proper error mapping via `ToolError::IntoResponse` (HTTP status derived from `kind()`)

## Not Yet Implemented

- Correspondent partial update (`PATCH /api/correspondents/{id}/`) — deferred
- Document type partial update (`PATCH /api/document_types/{id}/`) — deferred
- Saved view update/delete (`PATCH/DELETE /api/saved_views/{id}/`) — deferred
- Storage path update/delete (`PATCH/DELETE /api/storage_paths/{id}/`) — deferred
- Users / groups management — deferred
