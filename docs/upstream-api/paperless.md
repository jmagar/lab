# Paperless-ngx API Reference

> **No static OpenAPI spec is published.** Paperless-ngx uses `drf-spectacular` to generate the schema dynamically from the running instance.
>
> Source: served at `/api/schema/` on every Paperless instance.
> The demo (`https://demo.paperless-ngx.com/api/schema/`) blocks anonymous downloads (403).

## How to fetch the spec from your own instance

```bash
curl -fsSL \
  -H "Authorization: Token $PAPERLESS_API_KEY" \
  "$PAPERLESS_URL/api/schema/?format=openapi-json" \
  -o docs/api-specs/paperless.openapi.json
```

Or YAML:
```bash
curl -fsSL \
  -H "Authorization: Token $PAPERLESS_API_KEY" \
  "$PAPERLESS_URL/api/schema/" \
  -o docs/api-specs/paperless.openapi.yaml
```

The schema is also browsable at `$PAPERLESS_URL/api/schema/swagger-ui/` and `$PAPERLESS_URL/api/schema/redoc/`.

## Quick reference (high-level endpoint groups)

> Source: https://docs.paperless-ngx.com/api/

REST, token auth (`Authorization: Token <key>`). Base URL: `$PAPERLESS_URL/api/`.

### Documents
- `GET    /api/documents/` — list, supports `query`, `tags__id__all`, `correspondent__id`, `document_type__id`, `created__date__gt`, `ordering`, `page`, `page_size`
- `GET    /api/documents/{id}/` — single document
- `POST   /api/documents/post_document/` — multipart upload (`document` file, `title`, `created`, `correspondent`, `document_type`, `tags`)
- `PATCH  /api/documents/{id}/` — partial update
- `DELETE /api/documents/{id}/`
- `GET    /api/documents/{id}/download/` — original file
- `GET    /api/documents/{id}/preview/` — preview file
- `GET    /api/documents/{id}/thumb/` — thumbnail
- `GET    /api/documents/{id}/metadata/` — full metadata
- `POST   /api/documents/bulk_edit/` — bulk operations (set tags, set correspondent, delete, etc.)

### Tags
- `GET    /api/tags/` — list
- `POST   /api/tags/` — create (`name`, `color`, `match`, `matching_algorithm`, `is_inbox_tag`)
- `PATCH  /api/tags/{id}/`
- `DELETE /api/tags/{id}/`

### Correspondents
- `GET    /api/correspondents/` — list
- `POST   /api/correspondents/` — create (`name`, `match`, `matching_algorithm`)
- `PATCH  /api/correspondents/{id}/`
- `DELETE /api/correspondents/{id}/`

### Document Types
- `GET    /api/document_types/`
- `POST   /api/document_types/`
- `PATCH  /api/document_types/{id}/`
- `DELETE /api/document_types/{id}/`

### Storage Paths
- `GET    /api/storage_paths/`
- `POST   /api/storage_paths/`
- `PATCH  /api/storage_paths/{id}/`
- `DELETE /api/storage_paths/{id}/`

### Custom Fields
- `GET    /api/custom_fields/`
- `POST   /api/custom_fields/`

### Saved Views
- `GET    /api/saved_views/`
- `POST   /api/saved_views/`

### Tasks
- `GET    /api/tasks/` — async task status (used to track ingestion progress)

### Statistics & UI
- `GET    /api/statistics/` — total docs, inbox count, etc.
- `GET    /api/ui_settings/` — current user UI preferences

### Auth
- `POST   /api/token/` — exchange username/password for token (`username`, `password`)

## Implementation note for `lab-apis`

Once we have the live spec from a real instance, save it as `paperless.openapi.json` next to this file. The Paperless types in `lab-apis/src/paperless/types.rs` should be hand-written to match the spec — `drf-spectacular` schemas are accurate but verbose, and not all endpoints are needed by lab.
