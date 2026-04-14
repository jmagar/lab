# Apprise API Coverage

**Last updated:** 2026-04-13
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
- `APPRISE_TOKEN` — bearer token (optional; most apprise-api deployments are unauthenticated)

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

Note: `/add/{key}` accepts a JSON body with `{config, format}` fields. `/get/{key}` and
`/del/{key}` are POST per the upstream API spec (not GET/DELETE). `HttpClient` gained
`post_text_void` and `post_empty_get_text` helpers to support these patterns.

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
| `body` | string | yes (unless `payload` set) | Notification body |
| `urls` | string or string[] | no | One or more Apprise URLs (stateless mode only) |
| `title` | string | no | Optional title |
| `type` | string | no | `info`, `success`, `warning`, `failure` |
| `format` | string | no | `text`, `markdown`, `html` |
| `tag` | string | no | Tag filter; `"all"` matches all tagged URLs |
| `payload` | json | no | Full request body override (overrides all individual params) |

### `notify.key.send` Parameters

| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `key` | string | yes | Stored config key |
| `body` | string | yes (unless `payload` set) | Notification body |
| `title` | string | no | Optional title |
| `type` | string | no | `info`, `success`, `warning`, `failure` |
| `format` | string | no | `text`, `markdown`, `html` |
| `tag` | string | no | Tag filter |
| `payload` | json | no | Full request body override |

### `config.add` Parameters

| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `key` | string | yes | Config key to store the config under |
| `config` | string | yes | Raw YAML or text Apprise config content |
| `format` | string | no | `yaml` or `text` (default: `yaml`) |

### `config.get` Parameters

| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `key` | string | yes | Config key to retrieve |

Returns `{config: string}` wrapping the raw YAML/text blob.

### `config.delete` Parameters

| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `key` | string | yes | Config key to delete |

**Destructive** — MCP elicitation required; CLI requires `-y` / `--yes`.

### `config.urls` Parameters

| Param | Type | Required | Notes |
|-------|------|----------|-------|
| `key` | string | yes | Config key to list URLs for |

Returns `Vec<{url: string, tags: Vec<string>}>`.

### `server.details` Parameters

None. Returns a large `serde_json::Value` listing all loaded Apprise plugins. Schema varies
by Apprise version; treat as opaque.

## Surface Details

### CLI (`lab apprise`)

Tier 2 (dispatch-backed thin shim). Accepts `action` as positional arg and `--params` as a
JSON string. Defaults to `help` when no action is given.

```
lab apprise server.health
lab apprise notify.send --params '{"body":"hello","urls":["slack://..."]}'
lab apprise notify.key.send --params '{"key":"mygroup","body":"hello"}'
lab apprise config.add --params '{"key":"mygroup","config":"urls:\n- slack://...","format":"yaml"}'
lab apprise config.get --params '{"key":"mygroup"}'
lab apprise config.delete --params '{"key":"mygroup"}' --yes
lab apprise config.urls --params '{"key":"mygroup"}'
lab apprise server.details
```

`config.delete` is destructive — requires `-y` / `--yes` in non-interactive CLI use.

### MCP

Single tool `apprise`. Thin adapter forwarding to `crate::dispatch::apprise::dispatch`.
`config.delete` triggers MCP elicitation before executing.

### API (`POST /v1/apprise`)

Standard action+params envelope. Uses `AppState.clients.apprise` (pre-built at startup from
`APPRISE_URL` + `APPRISE_TOKEN`). `help` and `schema` are routed through the top-level
dispatch function; all other actions go through `dispatch_with_client`.

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
