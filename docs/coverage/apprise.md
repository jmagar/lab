# Apprise API Coverage

**Last updated:** 2026-04-14
**Source spec:** docs/upstream-api/apprise.md
**Format:** hand-scraped reference

## Summary

- This doc is a lightweight implementation aid, not a machine-generated contract.
- The source file remains the canonical endpoint reference for this service.

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and wired through SDK, dispatch, CLI, MCP, and API |
| ⬜ | Not implemented yet; rows are planning inventory only |

## Config

- `APPRISE_URL` — base URL (required)
- `APPRISE_TOKEN` — bearer token (optional; most apprise-api deployments are unauthenticated; auth strategy: `Bearer` token in `Authorization` header)

## SDK Surface (`lab-apis`)

| Method | Endpoint | Purpose |
|--------|----------|---------|
| `health()` | `GET /status` | Health probe — returns `()` on 200 |
| `notify(request)` | `POST /notify` | Stateless notification to caller-supplied URLs |
| `notify_key(key, request)` | `POST /notify/{key}` | Notification via a stored config key |
| `add_config(key, config, format)` | `POST /add/{key}` | Persist a YAML/text config blob under a key |
| `get_config(key)` | `POST /get/{key}` | Retrieve stored config blob as raw text |
| `delete_config(key)` | `POST /del/{key}` | Delete a stored config key |
| `get_urls(key)` | `GET /json/urls/{key}` | List URLs+tags stored under a key |
| `details()` | `GET /details` | Server details listing all loaded plugins |

**Patterns:** `/add/{key}` accepts a JSON body with `{config, format}` fields. `/get/{key}` and
`/del/{key}` are POST per the upstream API spec (not GET/DELETE). `HttpClient` provides `post_text_void` and `post_empty_get_text` helpers to support these patterns. Path segments are percent-encoded per RFC 3986.

## Action Catalog (dispatch layer)

The dispatch action names differ from the SDK method names. The catalog is in
`crates/lab/src/dispatch/apprise/catalog.rs`.

### Built-ins (every tool)

| Action | Params | Returns |
|--------|--------|---------|
| `help` | — | Catalog |
| `schema` | `action` (string, required) | Schema |

### Implemented Actions

| Action | SDK call | Destructive | MCP | CLI | API |
|--------|----------|-------------|-----|-----|-----|
| `server.health` | `health()` | no | ✅ | ✅ | ✅ |
| `notify.send` | `notify(request)` | no | ✅ | ✅ | ✅ |
| `notify.key.send` | `notify_key(key, request)` | no | ✅ | ✅ | ✅ |
| `config.add` | `add_config(key, config, format)` | no | ✅ | ✅ | ✅ |
| `config.get` | `get_config(key)` | no | ✅ | ✅ | ✅ |
| `config.delete` | `delete_config(key)` | **yes** | ✅ | ✅ | ✅ |
| `config.urls` | `get_urls(key)` | no | ✅ | ✅ | ✅ |
| `server.details` | `details()` | no | ✅ | ✅ | ✅ |

### `notify.send` Parameters

| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `body` | string | no (optional if `payload` override provided) | Notification body text |
| `urls` | string or array of strings | no | One or more Apprise URLs (stateless mode only) |
| `title` | string | no | Optional notification title |
| `type` | string | no | Message type: `info`, `success`, `warning`, or `failure` |
| `format` | string | no | Body format: `text`, `markdown`, or `html` |
| `tag` | string | no | Optional tag filter |
| `payload` | json | no | Full request body override (supersedes all individual params when provided) |

### `notify.key.send` Parameters

| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `key` | string | **yes** | Stored config key |
| `body` | string | no (optional if `payload` override provided) | Notification body text |
| `title` | string | no | Optional notification title |
| `type` | string | no | Message type: `info`, `success`, `warning`, or `failure` |
| `format` | string | no | Body format: `text`, `markdown`, or `html` |
| `tag` | string | no | Optional tag filter |
| `payload` | json | no | Full request body override (supersedes all individual params when provided) |

### `config.add` Parameters

| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `key` | string | **yes** | Config key to store the config under |
| `config` | string | **yes** | Raw YAML or text Apprise config content |
| `format` | string | no | Config format: `yaml` or `text` (default: `yaml`) |

### `config.get` Parameters

| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `key` | string | **yes** | Config key to retrieve |

Returns a JSON wrapper `{config: string}` containing the raw YAML/text blob.

### `config.delete` Parameters

| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `key` | string | **yes** | Config key to delete |

**Destructive** — MCP requires elicitation; CLI requires `-y` / `--yes`; API requires `confirm: true` in params.

### `config.urls` Parameters

| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `key` | string | **yes** | Config key to list URLs for |

Returns `Vec<{url: string, tags: Vec<string>}>`.

### `server.health` Parameters

None. Returns `{ok: true}` on success.

### `server.details` Parameters

None. Returns a large `serde_json::Value` listing all loaded Apprise plugins and their capabilities. Schema varies by Apprise version; treat as opaque.

## Surface Details

### CLI (`labby apprise`)

Tier 2 (dispatch-backed thin shim). Accepts `action` as positional arg and `--params` as a
JSON string. Defaults to `help` when no action is given.

```bash
labby apprise server.health
labby apprise notify.send --params '{"body":"hello","urls":["slack://..."]}'
labby apprise notify.key.send --params '{"key":"mygroup","body":"hello"}'
labby apprise config.add --params '{"key":"mygroup","config":"urls:\n- slack://...","format":"yaml"}'
labby apprise config.get --params '{"key":"mygroup"}'
labby apprise config.delete --params '{"key":"mygroup"}' --yes
labby apprise config.urls --params '{"key":"mygroup"}'
labby apprise server.details
```

`config.delete` is destructive — requires `-y` / `--yes` in non-interactive CLI use.

### MCP

Single tool `apprise`. Thin adapter forwarding to `crate::dispatch::apprise::dispatch` (shared dispatch layer). `config.delete` triggers MCP elicitation before executing.

### API (`POST /v1/apprise`)

Standard action+params envelope. Uses `AppState.clients.apprise` (pre-built at startup from
`APPRISE_URL` + `APPRISE_TOKEN`). Routes:

- `help` and `schema` are routed through the top-level `dispatch()` function (which does not require a live client).
- All other actions route through `dispatch_with_client()` with the pre-built client from `AppState`.

If the client is not configured (`APPRISE_URL` is missing or initialization failed), non-`help`/`schema` actions return HTTP 500 with `kind: "internal_error"` and a descriptive message.

## API Inventory

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /status | `health` | ✅ | ✅ | ✅ | ✅ |
| POST | /notify | `notify` | ✅ | ✅ | ✅ | ✅ |
| POST | /notify/{KEY} | `notify_key` | ✅ | ✅ | ✅ | ✅ |
| POST | /add/{KEY} | `add_config` | ✅ | ✅ | ✅ | ✅ |
| POST | /get/{KEY} | `get_config` | ✅ | ✅ | ✅ | ✅ |
| POST | /del/{KEY} | `delete_config` | ✅ | ✅ | ✅ | ✅ |
| GET | /json/urls/{KEY} | `get_urls` | ✅ | ✅ | ✅ | ✅ |
| GET | /details | `details` | ✅ | ✅ | ✅ | ✅ |
| GET | /metrics | `metrics` | ⬜ | ⬜ | ⬜ | ⬜ |
