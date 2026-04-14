# ByteStash API Coverage

**Last updated:** 2026-04-14
**Source spec:** `docs/upstream-api/bytestash.md`
**SDK surface:** `crates/lab-apis/src/bytestash/client.rs`
**Shared dispatch:** `crates/lab/src/dispatch/bytestash/` (catalog.rs, client.rs, params.rs, dispatch.rs)
**MCP adapter:** `crates/lab/src/mcp/services/bytestash.rs` (tests only; registry wires dispatch directly)
**CLI surface:** `crates/lab/src/cli/bytestash.rs` (generic `action` + `key=value` params → `run_confirmable_action_command`)
**API handler:** `crates/lab/src/api/services/bytestash.rs` (thin adapter over shared dispatch)

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented |
| ⬜ | Not implemented |
| ❌ | Removed — endpoint does not exist in ByteStash |
| — | Not applicable |

> **Auth note:** ByteStash uses a non-standard header `bytestashauth: Bearer <jwt>`.
> The client sends this via `Auth::ApiKey { header: "bytestashauth", key: "Bearer <token>" }`.
> `BYTESTASH_TOKEN` must be a valid JWT (obtained via `auth.login`), not a raw API key.
>
> API key auth is currently broken upstream — `authenticateToken` runs after
> `apiKeyAuth` sets `req.user` without short-circuiting, so API keys always 401.

## Config

- `BYTESTASH_URL` — base URL, e.g. `https://bytestash.tootie.tv` (required)
- `BYTESTASH_TOKEN` — JWT from `auth.login` (required; not an API key)

## SDK Surface (`lab-apis`)

| Method | Endpoint | Notes |
|--------|----------|-------|
| `probe()` | `GET /api/auth/config` | Health probe; returns `()` |
| `auth_config()` | `GET /api/auth/config` | Auth provider configuration |
| `auth_register(body)` | `POST /api/auth/register` | Register a new user |
| `auth_login(body)` | `POST /api/auth/login` | Log in and receive a JWT |
| `snippets_list()` | `GET /api/snippets` | List the caller's snippets |
| `snippet_get(id)` | `GET /api/snippets/:id` | Fetch one snippet |
| `snippets_create(body)` | `POST /api/snippets` | Create a snippet |
| `snippets_update(id, body)` | `PUT /api/snippets/:id` | Update a snippet |
| `snippets_delete(id)` | `DELETE /api/snippets/:id` | Delete a snippet |
| `snippets_public_list()` | `GET /api/public/snippets` | List public snippets |
| `snippets_public_get(id)` | `GET /api/public/snippets/:id` | Fetch one public snippet |
| `snippets_share_create(body)` | `POST /api/share` | Create a share link |
| `snippets_share_get(share_id)` | `GET /api/share/:share_id` | Fetch a shared snippet |
| `categories_list()` | derived from `GET /api/snippets` | Deduped categories; no dedicated endpoint |
| `users_list()` | `GET /api/admin/users` | List users (admin routes — newer ByteStash only) |
| `users_toggle_active(id)` | `PATCH /api/admin/users/:id/toggle-active` | Toggle user active status (admin only) |
| `users_delete(id)` | `DELETE /api/admin/users/:id` | Delete a user (admin only) |

## Action Catalog

All actions are dispatched via the shared dispatch layer in `crates/lab/src/dispatch/bytestash/`.
The catalog is the authoritative source of truth in `catalog.rs`. Every action listed below
is wired across all surfaces: SDK, dispatch, MCP, CLI, and API.

All actions are accessible via:
- **MCP:** `bytestash({ "action": "<action>", "params": {...} })`
- **CLI:** `lab bytestash <action> [key=value ...]`
- **API:** `POST /v1/bytestash { "action": "<action>", "params": {...}, "confirm": true/false }`

Destructive actions require `-y` / `--yes` flag on CLI or `"confirm": true` in API params.

### Built-in

| Action | Endpoint | SDK | Destructive | MCP | CLI | API | Notes |
|--------|----------|-----|-------------|-----|-----|-----|-------|
| `help` | — | — | no | ✅ | ✅ | ✅ | Show action catalog; always available |
| `schema` | — | — | no | ✅ | ✅ | ✅ | Return parameter schema for an action |

### Auth

| Action | Endpoint | SDK | Destructive | MCP | CLI | API |
|--------|----------|-----|-------------|-----|-----|-----|
| `auth.config` | `GET /api/auth/config` | ✅ | no | ✅ | ✅ | ✅ |
| `auth.register` | `POST /api/auth/register` | ✅ | **yes** | ✅ | ✅ | ✅ |
| `auth.login` | `POST /api/auth/login` | ✅ | no | ✅ | ✅ | ✅ |

### Snippets

| Action | Endpoint | SDK | Destructive | MCP | CLI | API |
|--------|----------|-----|-------------|-----|-----|-----|
| `snippets.list` | `GET /api/snippets` | ✅ | no | ✅ | ✅ | ✅ |
| `snippets.get` | `GET /api/snippets/:id` | ✅ | no | ✅ | ✅ | ✅ |
| `snippets.create` | `POST /api/snippets` | ✅ | **yes** | ✅ | ✅ | ✅ |
| `snippets.update` | `PUT /api/snippets/:id` | ✅ | **yes** | ✅ | ✅ | ✅ |
| `snippets.delete` | `DELETE /api/snippets/:id` | ✅ | **yes** | ✅ | ✅ | ✅ |

### Public / Shared Snippets

| Action | Endpoint | SDK | Destructive | MCP | CLI | API |
|--------|----------|-----|-------------|-----|-----|-----|
| `snippets.public.list` | `GET /api/public/snippets` | ✅ | no | ✅ | ✅ | ✅ |
| `snippets.public.get` | `GET /api/public/snippets/:id` | ✅ | no | ✅ | ✅ | ✅ |
| `snippets.share.create` | `POST /api/share` | ✅ | **yes** | ✅ | ✅ | ✅ |
| `snippets.share.get` | `GET /api/share/:share_id` | ✅ | no | ✅ | ✅ | ✅ |

### Categories

| Action | Endpoint | SDK | Destructive | MCP | CLI | API | Notes |
|--------|----------|-----|-------------|-----|-----|-----|-------|
| `categories.list` | derived from `GET /api/snippets` | ✅ | no | ✅ | ✅ | ✅ | No dedicated endpoint; deduped client-side |

### Users (admin — requires newer ByteStash with `/api/admin` routes)

| Action | Endpoint | SDK | Destructive | MCP | CLI | API |
|--------|----------|-----|-------------|-----|-----|-----|
| `users.list` | `GET /api/admin/users` | ✅ | no | ✅ | ✅ | ✅ |
| `users.toggle-active` | `PATCH /api/admin/users/:id/toggle-active` | ✅ | **yes** | ✅ | ✅ | ✅ |
| `users.delete` | `DELETE /api/admin/users/:id` | ✅ | **yes** | ✅ | ✅ | ✅ |

## Key Param Details

**`auth.register` and `auth.login`** (both accept alternative full-body override via `payload`):
- `username` (required) — username
- `password` (required) — password
- `payload` (optional) — full JSON body (overrides individual params)

**`snippets.list`** — no params

**`snippets.get`** (requires `id`):
- `id` (required) — snippet ID

**`snippets.create`** (accepts alternative full-body override via `payload`):
- `title` (required) — snippet title
- `description` (optional) — optional description
- `language` (optional) — optional language label
- `fragments` (optional, JSON) — snippet fragments JSON
- `categories` (optional, JSON) — category list JSON
- `payload` (optional) — full JSON body (overrides individual params)

**`snippets.update`** (accepts alternative full-body override via `payload`):
- `id` (required) — snippet ID
- `title` (optional) — snippet title
- `description` (optional) — optional description
- `language` (optional) — optional language label
- `fragments` (optional, JSON) — snippet fragments JSON
- `categories` (optional, JSON) — category list JSON
- `payload` (optional) — full JSON body (overrides individual params)

**`snippets.delete`** (requires `id`):
- `id` (required) — snippet ID

**`snippets.public.list`** — no params

**`snippets.public.get`** (requires `id`):
- `id` (required) — public snippet ID

**`snippets.share.create`** (accepts alternative full-body override via `payload`):
- `snippetId` (required) — snippet ID to share
- `requiresAuth` (optional, bool) — whether the link requires auth to view
- `expiresIn` (optional, integer) — expiry in seconds (null = never)
- `payload` (optional) — full JSON body (overrides individual params)

**`snippets.share.get`** (requires `share_id`):
- `share_id` (required) — share link ID

**`categories.list`** — no params

**`users.list`** — no params

**`users.toggle-active`** (requires `id`):
- `id` (required) — user ID

**`users.delete`** (requires `id`):
- `id` (required) — user ID

## Surface Details

### SDK (`lab-apis/src/bytestash/`)

Pure Rust client library. Types live in `types.rs`, client methods in `client.rs`, service-specific
errors in `error.rs`. All HTTP methods implemented: `probe()` for health checks, plus 13 business
methods (auth, snippets, categories, users admin). Returns `Result<Value, ByteStashError>` where
`Value` is a `serde_json::Value` for flexibility across heterogeneous responses.

### Shared Dispatch (`crates/lab/src/dispatch/bytestash/`)

Migrated to the standard 4-file layout:

- **`catalog.rs`** — 19 `ActionSpec` definitions (help, schema, 17 service actions) — source of truth
- **`client.rs`** — client resolution: `require_client()` for dispatch/CLI, `client_from_env()` for startup
- **`params.rs`** — parameter coercion and validation helpers (credentials, snippet writes, share requests)
- **`dispatch.rs`** — action routing and client dispatch; handles help/schema built-ins

The dispatcher returns surface-neutral `Result<Value, ToolError>` consumed by MCP, CLI, and API.

### CLI (`crates/lab/src/cli/bytestash.rs`)

Tier 2 dispatch-backed thin shim. Accepts positional `action` and trailing `key=value` params.
Supports `-y` / `--yes` (alias `--no-confirm`) and `--dry-run`.

```bash
lab bytestash help
lab bytestash snippets.list
lab bytestash snippets.get id=abc123
lab bytestash snippets.create title="My snippet" language=rust
lab bytestash snippets.delete id=abc123 -y
lab bytestash auth.login username=admin password=secret
```

Destructive actions (registered in the catalog) require `-y` or `--dry-run` on non-TTY or TTY without `-y`.

### MCP

Single tool: `bytestash`. The `mcp/services/bytestash.rs` file contains only tests; the registry
in `mcp/registry.rs` wires `crate::dispatch::bytestash::dispatch` directly (not a thin adapter).

Tool schema and action catalog are auto-generated from `dispatch/bytestash/catalog.rs` via `ACTIONS`.

```
bytestash({ "action": "snippets.list", "params": {} })
bytestash({ "action": "snippets.create", "params": { "title": "...", "language": "rust" } })
bytestash({ "action": "help", "params": {} })
bytestash({ "action": "schema", "params": { "action": "snippets.create" } })
```

### API (`POST /v1/bytestash`)

Standard action+params envelope. Route mounted at `POST /v1/bytestash` in `api/services/bytestash.rs`.
Uses `AppState.clients.bytestash` (pre-built at startup from `BYTESTASH_URL` + `BYTESTASH_TOKEN`).

Destructive actions require `"confirm": true` in params (boolean, not string).

```json
POST /v1/bytestash
{
  "action": "snippets.create",
  "params": {
    "title": "My snippet",
    "language": "rust",
    "confirm": true
  }
}
```

## Implementation Notes

### Architecture

The bytestash service is fully migrated to the shared dispatch layer (`crates/lab/src/dispatch/bytestash/`).
All 19 actions (17 service-level + 2 built-ins) are defined in `catalog.rs` and routed through
`dispatch.rs` to SDK client methods in `lab-apis`. The dispatcher returns `Result<Value, ToolError>`,
which is then formatted for display by the surface layer (CLI, MCP, or HTTP API).

### Configuration

- `BYTESTASH_URL` — base URL, e.g. `https://bytestash.tootie.tv` (required)
- `BYTESTASH_TOKEN` — JWT from `auth.login` (required; **not** a static API key)

Loaded from `~/.lab/.env`. Both must be present; if either is missing, the dispatcher returns
`internal_error` with a clear message.

### Auth

ByteStash uses a **non-standard auth header**: `bytestashauth: Bearer <jwt>`.

The client constructs this via `Auth::ApiKey { header: "bytestashauth", key: format!("Bearer {token}") }`.
This is wired in `dispatch/bytestash/client.rs` and applies to all API calls.

### Client Resolution

- **Startup (AppState):** `dispatch::bytestash::client_from_env()` — called by `AppState::ServiceClients::from_env()` at startup
- **MCP/CLI:** `dispatch::bytestash::client::require_client()` — fallback when called without AppState; reads env and propagates errors
- **API:** Pre-built client from `AppState.clients.bytestash` — passed to `dispatch_with_client()`

### Notable Behaviors

- `categories.list` has no server-side endpoint — the dispatcher fetches all snippets and
  deduplicates the category strings locally.
- Admin user endpoints (`users.*`) require a ByteStash deployment with `/api/admin` routes.
  These are typically available in newer versions; older deployments (like `bytestash.tootie.tv`)
  may not expose them and will return 404.
- `snippets.delete` and `users.delete` return `void` (HTTP 204 No Content); the dispatcher
  returns `Value::Null` to callers.
- Destructive actions are marked in the catalog and enforced at the surface layer: CLI requires
  `-y` / `--yes`, API requires `"confirm": true` in params, MCP triggers elicitation.

### Test Coverage

The dispatch layer has unit tests in `dispatch/bytestash.rs`:
- Action catalog completeness
- Destructive action metadata
- Unknown action rejection
- Help and schema built-ins
- Client resolution from env vars and explicit values
