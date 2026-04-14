# Memos API Coverage

**Last updated:** 2026-04-13
**API target:** Memos v1 REST API (resource-name style, RFC3339 timestamps)
**SDK surface:** `crates/lab-apis/src/memos/client.rs` (18 public methods)
**Shared dispatch:** `crates/lab/src/dispatch/memos.rs` + `crates/lab/src/dispatch/memos/` (catalog, client, params, dispatch)
**MCP adapter:** `crates/lab/src/mcp/services/memos.rs` (thin re-export of dispatch layer)
**CLI surface:** `crates/lab/src/cli/memos.rs` (generic `action` + `--params JSON` → shared dispatch)
**API handler:** `crates/lab/src/api/services/memos.rs` (thin adapter over shared dispatch)

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and live-tested |
| ⬜ | Implemented, not yet live-tested |
| — | Not implemented (out of scope for initial cut) |

> **Auth note:** Memos uses `Authorization: Bearer <token>`.
> Env vars: `MEMOS_URL`, `MEMOS_TOKEN`.

## SDK Surface

| Method | Endpoint | Purpose |
|--------|----------|---------|
| `health()` | `GET /api/v1/workspace/profile` | Health probe (re-uses workspace profile) |
| `memos_list(params)` | `GET /api/v1/memos` | List memos with optional filters |
| `memo_get(name)` | `GET /api/v1/{name}` | Get a single memo by resource name |
| `memo_create(req)` | `POST /api/v1/memos` | Create a new memo |
| `memo_update(name, req)` | `PATCH /api/v1/{name}` | Update a memo by resource name |
| `memo_delete(name)` | `DELETE /api/v1/{name}` | Delete a memo by resource name |
| `tags_list()` | `GET /api/v1/tags` | List all tags |
| `workspace_profile()` | `GET /api/v1/workspace/profile` | Get workspace profile |
| `user_me()` | `GET /api/v1/users/me` | Get authenticated user profile |
| `users_list()` | `GET /api/v1/users` | List all users (admin only) |
| `user_stats(user)` | `GET /api/v1/{user}:getStats` | Get memo statistics for a user |
| `webhooks_list(user)` | `GET /api/v1/{user}/webhooks` | List webhooks for a user |
| `webhook_create(user, req)` | `POST /api/v1/{user}/webhooks` | Create a webhook for a user |
| `attachment_upload(filename, bytes, mime)` | `POST /api/v1/attachments` | Upload a file attachment (multipart) |
| `attachment_delete(name)` | `DELETE /api/v1/{name}` | Delete an attachment |
| `memo_comments_list(memo_name)` | `GET /api/v1/{name}/comments` | List comments on a memo |
| `memo_comment_create(memo_name, req)` | `POST /api/v1/{name}/comments` | Create a comment on a memo |
| `memo_shares_list(memo_name)` | `GET /api/v1/{name}/shares` | List share links for a memo |
| `memo_share_create(memo_name)` | `POST /api/v1/{name}/shares` | Create a share link for a memo |

## Action Catalog

All actions are implemented in the shared dispatch layer and wired to MCP, CLI, and API surfaces.

### Memos

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `memos.list` | `GET /api/v1/memos` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `memos.get` | `GET /api/v1/{name}` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `memos.create` | `POST /api/v1/memos` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `memos.update` | `PATCH /api/v1/{name}` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `memos.delete` | `DELETE /api/v1/{name}` | ✅ | ✅ | ✅ | ⬜ | ⬜ |

### Tags

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `tags.list` | `GET /api/v1/tags` | ✅ | ✅ | ✅ | ⬜ | ⬜ |

### Workspace

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `workspace.profile` | `GET /api/v1/workspace/profile` | ✅ | ✅ | ✅ | ⬜ | ⬜ |

### User

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `user.me` | `GET /api/v1/users/me` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `user.list` | `GET /api/v1/users` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `user.stats` | `GET /api/v1/{user}:getStats` | ✅ | ✅ | ✅ | ⬜ | ⬜ |

### Webhooks

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `webhook.list` | `GET /api/v1/{user}/webhooks` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `webhook.create` | `POST /api/v1/{user}/webhooks` | ✅ | ✅ | ✅ | ⬜ | ⬜ |

### Attachments

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `attachment.upload` | `POST /api/v1/attachments` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `attachment.delete` | `DELETE /api/v1/{name}` | ✅ | ✅ | ✅ | ⬜ | ⬜ |

### Memo Sub-Resources

| Action | Endpoint | SDK | Dispatch | MCP | CLI | API |
|--------|----------|-----|----------|-----|-----|-----|
| `memo.comment-list` | `GET /api/v1/{name}/comments` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `memo.comment-create` | `POST /api/v1/{name}/comments` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `memo.share-list` | `GET /api/v1/{name}/shares` | ✅ | ✅ | ✅ | ⬜ | ⬜ |
| `memo.share-create` | `POST /api/v1/{name}/shares` | ✅ | ✅ | ✅ | ⬜ | ⬜ |

## Action Parameters

### `memos.list`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `creator` | string | no | Filter by creator resource name, e.g. `"users/1"` |
| `tag` | string | no | Filter by tag name |
| `page_size` | integer | no | Maximum number of memos to return per page |
| `page_token` | string | no | Page token from a previous list response |

### `memos.get`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `name` | string | yes | Memo resource name, e.g. `"memos/123"` |

### `memos.create`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `content` | string | yes | Markdown content of the memo |
| `visibility` | string | no | `"PRIVATE"` (default), `"PROTECTED"`, or `"PUBLIC"` |

### `memos.update`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `name` | string | yes | Memo resource name, e.g. `"memos/123"` |
| `content` | string | no | New markdown content |
| `visibility` | string | no | New visibility: `"PRIVATE"`, `"PROTECTED"`, or `"PUBLIC"` |

### `memos.delete`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `name` | string | yes | Memo resource name, e.g. `"memos/123"` |

### `user.stats`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `user` | string | yes | User resource name, e.g. `"users/1"` or `"users/me"` |

### `webhook.list`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `user` | string | yes | User resource name, e.g. `"users/1"` or `"users/me"` |

### `webhook.create`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `user` | string | yes | User resource name, e.g. `"users/1"` or `"users/me"` |
| `url` | string | yes | Webhook target URL |
| `name` | string | yes | Display name for the webhook |

### `attachment.upload`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `filename` | string | yes | Original filename, e.g. `"photo.png"` |
| `data_base64` | string | yes | Base64-encoded file content (standard or URL-safe, with or without padding) |
| `mime_type` | string | yes | MIME type, e.g. `"image/png"` or `"application/pdf"` |

### `attachment.delete`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `name` | string | yes | Attachment resource name, e.g. `"attachments/123"` |

### `memo.comment-list`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `name` | string | yes | Memo resource name, e.g. `"memos/123"` |

### `memo.comment-create`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `name` | string | yes | Memo resource name, e.g. `"memos/123"` |
| `content` | string | yes | Comment text content |

### `memo.share-list`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `name` | string | yes | Memo resource name, e.g. `"memos/123"` |

### `memo.share-create`
| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `name` | string | yes | Memo resource name, e.g. `"memos/123"` |

## Destructive Actions

| Action | Confirmation (CLI) | Confirmation (API) |
|--------|-------------------|--------------------|
| `memos.delete` | no `-y` flag (CLI uses `run_action_command`) | `"confirm": true` in params |
| `attachment.delete` | no `-y` flag (CLI uses `run_action_command`) | `"confirm": true` in params |

> **Note:** The Memos CLI shim uses `run_action_command` (not `run_confirmable_action_command`),
> so it does not require a `-y` flag at the CLI level. Destructive confirmation applies to the API surface only.

## CLI Interface

The CLI uses a generic dispatch shim (Tier 2), not typed subcommands:

```bash
lab memos <action> [--params '<json>']
lab memos memos.list
lab memos memos.list --params '{"tag":"rust","page_size":20}'
lab memos memos.get --params '{"name":"memos/123"}'
lab memos memos.create --params '{"content":"Hello world","visibility":"PRIVATE"}'
lab memos memos.update --params '{"name":"memos/123","content":"Updated content"}'
lab memos memos.delete --params '{"name":"memos/123"}'
lab memos tags.list
lab memos workspace.profile
lab memos user.me
```

## Upstream API Endpoints Not Implemented

The following Memos v1 API endpoints exist upstream but are not implemented in the SDK or dispatch layer:

### AttachmentService
| Method | Endpoint |
|--------|----------|
| GET | /api/v1/attachments |
| POST | /api/v1/attachments |
| DELETE | /api/v1/attachments/{attachment} |
| GET | /api/v1/attachments/{attachment} |
| PATCH | /api/v1/attachments/{attachment} |
| POST | /api/v1/attachments:batchDelete |

### AuthService
| Method | Endpoint |
|--------|----------|
| GET | /api/v1/auth/me |
| POST | /api/v1/auth/refresh |
| POST | /api/v1/auth/signin |
| POST | /api/v1/auth/signout |

### IdentityProviderService
| Method | Endpoint |
|--------|----------|
| GET/POST | /api/v1/identity-providers |
| GET/PATCH/DELETE | /api/v1/identity-providers/{identity-provider} |

### InstanceService
| Method | Endpoint |
|--------|----------|
| GET | /api/v1/instance/profile |
| GET/PATCH | /api/v1/instance/{instance}/* |

### MemoService (partial — sub-resources not implemented)
| Method | Endpoint |
|--------|----------|
| GET/PATCH | /api/v1/memos/{memo}/attachments |
| GET/POST | /api/v1/memos/{memo}/comments |
| GET/POST | /api/v1/memos/{memo}/reactions |
| DELETE | /api/v1/memos/{memo}/reactions/{reaction} |
| GET/PATCH | /api/v1/memos/{memo}/relations |
| GET/POST | /api/v1/memos/{memo}/shares |
| DELETE | /api/v1/memos/{memo}/shares/{share} |
| GET | /api/v1/shares/{shareId} |

### ShortcutService
| Method | Endpoint |
|--------|----------|
| GET/POST | /api/v1/users/{user}/shortcuts |
| GET/PATCH/DELETE | /api/v1/users/{user}/shortcuts/{shortcut} |

### UserService (partial — only `users/me` is implemented)
| Method | Endpoint |
|--------|----------|
| GET/POST | /api/v1/users |
| GET/PATCH/DELETE | /api/v1/users/{user} |
| GET/DELETE/PATCH | /api/v1/users/{user}/notifications/{notification} |
| GET/POST | /api/v1/users/{user}/personalAccessTokens |
| DELETE | /api/v1/users/{user}/personalAccessTokens/{personalAccessToken} |
| GET | /api/v1/users/{user}/settings |
| GET/PATCH | /api/v1/users/{user}/settings/{setting} |
| GET/POST | /api/v1/users/{user}/webhooks |
| PATCH/DELETE | /api/v1/users/{user}/webhooks/{webhook} |
| GET | /api/v1/users/{user}:getStats |
| POST | /api/v1/users:batchGet |
| GET | /api/v1/users:stats |

## Notes

- `MEMOS_URL` must point at the service root, e.g. `http://memos.home:5230`.
- `MEMOS_TOKEN` is a personal access token from the Memos settings page.
- Memo identifiers are resource names (`"memos/123"`), not bare integers.
- `memos.update` uses PATCH semantics — only provided fields are changed.
- The `health()` SDK method calls `workspace_profile` internally; it is used by `lab doctor` and not exposed as a separate action.
