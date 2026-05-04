# HF TEI API Coverage

**Last updated:** 2026-04-14
**OpenAPI spec:** docs/upstream-api/tei.openapi.json
**OpenAPI version:** 3.0.3
**API version:** 1.9.3
**Paths:** 12

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and wired through SDK, dispatch, CLI, MCP, and API |
| ⬜ | Not implemented yet; rows are spec inventory only |
| - | Not applicable / not represented in the spec |

Per-surface coverage: each column (MCP, CLI, API) tracks whether that surface has a working
implementation for this method. A method with ✅ in SDK but ⬜ in a surface column is
implemented in the client but not yet wired through that surface.

The source spec is the contract. This document is the implementation planning aid.

## SDK Surface (`crates/lab-apis/src/tei/client.rs`)

| Method | Endpoint | SDK Method | Impl |
|--------|----------|------------|------|
| POST | /embed | `embed()` | ✅ |
| GET | /health | `health()` | ✅ |
| GET | /info | `model_info()` | ✅ |
| POST | /embed_sparse | `embed_sparse()` | ✅ |
| POST | /rerank | `rerank()` | ✅ |
| POST | /similarity | `similarity()` | ✅ |
| POST | /tokenize | `tokenize()` | ✅ |
| POST | /v1/embeddings | `openai_embed()` | ✅ |
| POST | /decode | — | ⬜ |
| POST | /embed_all | — | ⬜ |
| GET | /metrics | — | ⬜ |
| POST | /predict | — | ⬜ |

## Dispatch Actions (`crates/lab/src/dispatch/tei/catalog.rs`)

The dispatch layer exposes ten actions. The built-in `help` and `schema` actions are also available on all surfaces.

| Action | SDK Method | Params | Destructive | Returns |
|--------|------------|--------|-------------|---------|
| `help` | — | none | No | Catalog |
| `schema` | — | `action: string` (required) | No | Schema |
| `server.health` | `health()` | none | No | `void` |
| `server.info` | `model_info()` | none | No | `Info` |
| `embed.create` | `embed()` | `input: string` (optional); `inputs: json` (optional); `normalize: bool` (optional); `truncate: bool` (optional); `payload: json` (optional) | No | `number[][]` |
| `embed.rerank` | `rerank()` | `query: string` (required); `texts: json` (required, max 100); `truncate: bool` (optional); `raw_scores: bool` (optional) | No | `RerankHit[]` |
| `embed.tokenize` | `tokenize()` | `inputs: json` (required); `add_special_tokens: bool` (optional) | No | `u32[][]` |
| `embed.similarity` | `similarity()` | `inputs: json` (required) | No | `f32[]` |
| `embed.sparse` | `embed_sparse()` | `inputs: json` (required); `truncate: bool` (optional) | No | `SparseToken[][]` |
| `embed.openai` | `openai_embed()` | `body: json` (required) | No | `json` |

### `embed.create` param resolution

The dispatcher (`crates/lab/src/dispatch/tei/params.rs`) resolves inputs with the following priority:

1. `payload` — full request body object; when present, all other keys are ignored except that `payload` itself is taken as the source.
2. `inputs` — string or array of strings.
3. `input` — single string shortcut (used only when `inputs` is absent or empty).

At least one of `input`, `inputs`, or `payload.inputs` must be present; missing inputs
returns a `MissingParam` error with `param: "inputs"`.

### `embed.rerank` rate limit

The dispatch layer enforces a hard cap of 100 texts per call. Requests with `texts.len() > 100`
return a `validation_error` with message indicating the limit and recommending splitting into multiple calls of ≤100 texts.

## Surface Coverage

| Action | MCP | CLI | API |
|--------|-----|-----|-----|
| `server.health` | ✅ | ✅ | ✅ |
| `server.info` | ✅ | ✅ | ✅ |
| `embed.create` | ✅ | ✅ | ✅ |
| `embed.rerank` | ✅ | ✅ | ✅ |
| `embed.tokenize` | ✅ | ✅ | ✅ |
| `embed.similarity` | ✅ | ✅ | ✅ |
| `embed.sparse` | ✅ | ✅ | ✅ |
| `embed.openai` | ✅ | ✅ | ✅ |

### CLI (`crates/lab/src/cli/tei.rs`)

Tier-2 shim. `labby tei <action> [--params '<json>']`. The `action` argument defaults to `help`
when omitted. All eight actions plus `help` and `schema` are wired.

### MCP (`crates/lab/src/registry.rs`)

Thin bridge delegating to `crate::dispatch::tei::dispatch()`. One MCP tool named `tei`.
Exports `ACTIONS` from the shared dispatch catalog and `dispatch` function for the MCP registry.

### API (`crates/lab/src/api/services/tei.rs`)

`POST /v1/tei` — single route. The route handler:

1. For `help` and `schema` actions: calls `crate::dispatch::tei::dispatch()` directly (which does not require a client)
2. For all other actions: builds a client from `AppState.clients.tei` (initialized at startup from `TEI_URL` and optional `TEI_API_KEY`) and calls `crate::dispatch::tei::dispatch_with_client()`.

If the client is not configured (missing `TEI_URL`), returns `internal_error` with message "TEI_URL not configured".

## Client Construction

The dispatcher layer (`crates/lab/src/dispatch/tei/client.rs`) handles client creation:

- `require_client()` — looks up `TEI_URL` (required) and `TEI_API_KEY` (optional) from env vars. Returns a structured `internal_error` if `TEI_URL` is absent or if TLS init fails.
- `client_from_env()` — used by `AppState` at startup; returns `Option<TeiClient>` for graceful missing-config handling.

Authentication:

- `TEI_URL` is always required.
- `TEI_API_KEY` (if set) is sent as `Authorization: Bearer <token>`.
- When `TEI_API_KEY` is absent or empty, no auth header is sent.

## Config

| Env var | Required | Purpose |
|---------|----------|---------|
| `TEI_URL` | Yes | Base URL of the TEI server |
| `TEI_API_KEY` | No | Bearer token (`Authorization: Bearer <key>`). Omit for unauthenticated local instances. |

## Unimplemented Spec Paths

The following paths from the OpenAPI spec have no SDK method, dispatch action, or surface wiring:

- `POST /decode`
- `POST /embed_all`
- `GET /metrics`
- `POST /predict`
