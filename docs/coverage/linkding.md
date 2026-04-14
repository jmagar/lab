# Linkding API Coverage

**Last updated:** 2026-04-13
**Source spec:** `docs/upstream-api/linkding.md`
**SDK surface:** `crates/lab-apis/src/linkding/client.rs` (20 public methods: 19 wrappers + `probe()`)
**Shared dispatch:** `crates/lab/src/dispatch/linkding.rs` + `crates/lab/src/dispatch/linkding/` (catalog, client, params, dispatch)
**MCP adapter:** `crates/lab/src/mcp/services/linkding.rs` (tests-only file; catalog and dispatch live in shared dispatch layer)
**CLI surface:** `crates/lab/src/cli/linkding.rs` (generic `action` + `--params JSON` → shared dispatch)
**API handler:** `crates/lab/src/api/services/linkding.rs` (thin adapter over shared dispatch)

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and live-tested |
| ⬜ | Implemented, not yet live-tested |
| — | Not applicable |

> **Auth note:** Linkding uses `Authorization: Token <api_token>`.
> The token is the REST API token from the Linkding Settings page.
> Env vars: `LINKDING_URL`, `LINKDING_TOKEN`.

## Live Test Evidence

Live smoke tests run 2026-04-12 against `<LINKDING_URL>`.

| Surface | Command | Result |
|---------|---------|--------|
| CLI | `lab linkding bookmarks.list` | bookmarks returned (live count) |
| CLI | `lab linkding tags.list` | tags returned (live count) |
| CLI | `lab linkding user.profile` | profile loaded |
| MCP | `mcporter call lab.linkding action=bookmarks.list` | `ok=true`, bookmarks returned |
| MCP | `mcporter call lab.linkding action=tags.list` | `ok=true`, tags returned |
| API | `POST /v1/linkding {"action":"bookmarks.list"}` | bookmarks returned |
| API | `POST /v1/linkding {"action":"tags.list"}` | tags returned |

## SDK Surface

| Method | Purpose |
|--------|---------|
| `probe()` | Health probe via `GET /api/bookmarks/?limit=1` |
| `bookmarks_list(params)` | List bookmarks with optional search/pagination |
| `bookmarks_archived_list(params)` | List archived bookmarks |
| `bookmark_get(id)` | Retrieve a single bookmark by ID |
| `bookmark_check(url)` | Check if a URL is already bookmarked |
| `bookmark_create(body)` | Create a new bookmark |
| `bookmark_update(id, body)` | Partially update a bookmark (PATCH) |
| `bookmark_archive(id)` | Archive a bookmark |
| `bookmark_unarchive(id)` | Unarchive a bookmark |
| `bookmark_delete(id)` | Delete a bookmark by ID |
| `tags_list()` | List all tags |
| `tag_get(id)` | Retrieve a single tag by ID |
| `tag_create(body)` | Create a new tag |
| `user_profile()` | Retrieve user profile / preferences |
| `bundles_list()` | List all bundles (saved searches) |
| `bundle_create(body)` | Create a new bundle |
| `bundle_update(id, patch)` | Partially update a bundle (PATCH) |
| `bundle_delete(id)` | Delete a bundle by ID |
| `bookmark_assets(id)` | List assets attached to a bookmark |
| `bookmark_assets_upload(id, name, bytes)` | Upload an asset for a bookmark (multipart) |

## Action Catalog

All actions are implemented in the shared dispatch layer and wired to MCP, CLI, and API surfaces.

### Bookmarks

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `bookmarks.list` | `GET /api/bookmarks/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `bookmarks.archived.list` | `GET /api/bookmarks/archived/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `bookmarks.get` | `GET /api/bookmarks/:id/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `bookmarks.check` | `GET /api/bookmarks/check/?url=` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `bookmarks.create` | `POST /api/bookmarks/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `bookmarks.update` | `PATCH /api/bookmarks/:id/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `bookmarks.archive` | `POST /api/bookmarks/:id/archive/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `bookmarks.unarchive` | `POST /api/bookmarks/:id/unarchive/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `bookmarks.delete` | `DELETE /api/bookmarks/:id/` | ✅ | ✅ | ✅ | ✅ | ✅ |

### Tags

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `tags.list` | `GET /api/tags/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `tags.get` | `GET /api/tags/:id/` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `tags.create` | `POST /api/tags/` | ✅ | ✅ | ✅ | ✅ | ✅ |

### User

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `user.profile` | `GET /api/user/profile/` | ✅ | ✅ | ✅ | ✅ | ✅ |

### Bundles

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `bundle.list` | `GET /api/bundles/` | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| `bundle.create` | `POST /api/bundles/` | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| `bundle.update` | `PATCH /api/bundles/:id/` | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| `bundle.delete` | `DELETE /api/bundles/:id/` | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |

### Bookmark Assets

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `bookmark.assets` | `GET /api/bookmarks/:id/assets/` | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |
| `bookmark.assets-upload` | `POST /api/bookmarks/:id/assets/upload/` | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ |

## Action Parameters

### `bookmarks.list` / `bookmarks.archived.list`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `q` | string | no | Search phrase |
| `limit` | integer | no | Maximum number of results (default 100) |
| `offset` | integer | no | Index from which to start returning results |

### `bookmarks.get` / `bookmarks.archive` / `bookmarks.unarchive` / `bookmarks.delete`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Bookmark ID |

### `bookmarks.check`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `url` | string | yes | URL to check |

### `bookmarks.create`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `url` | string | yes | Bookmark URL |
| `title` | string | no | Optional title (auto-scraped if omitted) |
| `description` | string | no | Optional description |
| `notes` | string | no | Optional markdown notes |
| `is_archived` | bool | no | Save directly to archive |
| `unread` | bool | no | Mark as unread |
| `shared` | bool | no | Share publicly |
| `tag_names` | json | no | Array of tag names to assign |
| `payload` | json | no | Alternative: full JSON body (overrides individual params) |

### `bookmarks.update`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Bookmark ID |
| `url` | string | no | New URL |
| `title` | string | no | New title |
| `description` | string | no | New description |
| `notes` | string | no | New notes |
| `unread` | bool | no | Update unread status |
| `shared` | bool | no | Update shared status |
| `tag_names` | json | no | Replace tag names (full replacement, not append) |
| `payload` | json | no | Alternative: full JSON body (overrides individual params) |

### `tags.get`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Tag ID |

### `tags.create`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `name` | string | yes | Tag name |
| `payload` | json | no | Alternative: full JSON body (overrides individual params) |

## Action Parameters

### `bundle.list`
No parameters.

### `bundle.create`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `name` | string | yes | Bundle name |
| `search_query` | string | yes | Linkding search query that populates this bundle |
| `description` | string | no | Optional description |

### `bundle.update`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Bundle ID |
| `payload` | json | yes | JSON object with fields to update (`name`, `search_query`, `description`) |

### `bundle.delete`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Bundle ID |

### `bookmark.assets`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Bookmark ID |

### `bookmark.assets-upload`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `id` | integer | yes | Bookmark ID |
| `file_name` | string | yes | Filename to use for the uploaded asset |
| `file_base64` | string | yes | Base64-encoded file content to upload |

## Destructive Actions

The following actions are marked `destructive: true` in the catalog:

| Action | Confirmation (CLI) | Confirmation (API) |
|--------|-------------------|--------------------|
| `bookmarks.archive` | no `-y` flag (CLI uses `run_action_command`) | `"confirm": true` in params |
| `bookmarks.delete` | no `-y` flag (CLI uses `run_action_command`) | `"confirm": true` in params |
| `tags.create` | no `-y` flag (CLI uses `run_action_command`) | `"confirm": true` in params |
| `bundle.delete` | no `-y` flag (CLI uses `run_action_command`) | `"confirm": true` in params |

> **Note:** The Linkding CLI shim uses `run_action_command` (not `run_confirmable_action_command`),
> so it does not require a `-y` flag at the CLI level. Destructive confirmation applies to the API surface only.

## CLI Interface

The CLI uses a generic dispatch shim (Tier 2), not typed subcommands:

```bash
lab linkding <action> [--params '<json>']
lab linkding bookmarks.list
lab linkding bookmarks.list --params '{"q":"rust","limit":10}'
lab linkding bookmarks.create --params '{"url":"https://example.com","title":"Example"}'
lab linkding tags.list
lab linkding user.profile
lab linkding bundle.list
lab linkding bundle.create --params '{"name":"Rust reads","search_query":"#rust"}'
lab linkding bundle.update --params '{"id":1,"payload":{"name":"Updated"}}'
lab linkding bundle.delete --params '{"id":1}'
lab linkding bookmark.assets --params '{"id":42}'
lab linkding bookmark.assets-upload --params '{"id":42,"file_name":"page.pdf","file_base64":"<base64>"}'
```

## Notes

- `LINKDING_URL` must point at the service root, e.g. `http://linkding.local:9090`.
- `LINKDING_TOKEN` is the REST API token from the Linkding Settings page (not a user password).
- Auth header: `Authorization: Token <token>` (standard Django REST Framework token auth).
- `bookmarks.update` uses PATCH semantics — only provided fields are changed.
- `bookmarks.create` auto-scrapes title/description from the URL if not provided.
- `bookmarks.unarchive` is **not** marked destructive (reversible operation).
- `bundle.update` uses PATCH semantics — pass a `payload` JSON object with only the fields to change.
- `bookmark.assets-upload` accepts `file_base64` (standard base64) as the file content; the dispatch layer decodes it before sending multipart/form-data to Linkding.
- `bundle.delete` is the only new destructive action.
