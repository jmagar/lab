# HF TEI API Coverage

**Last updated:** 2026-04-13
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

The dispatch layer exposes eight actions. The built-in `help` and `schema` actions are also available on all surfaces.

| Action | SDK Method | Params | Destructive | Returns |
|--------|------------|--------|-------------|---------|
| `help` | — | none | No | Catalog |
| `schema` | — | `action: string` (required) | No | Schema |
| `server.health` | `health()` | none | No | `{ ok: true }` |
| `server.info` | `model_info()` | none | No | `Info` |
| `embed.create` | `embed()` | `inputs: string[]` or `input: string` (required); `normalize: bool` (optional); `truncate: bool` (optional); `payload: object` (full body override, optional) | No | `number[][]` |
| `embed.rerank` | `rerank()` | `query: string` (required); `texts: string[]` (required, max 100); `truncate: bool` (optional); `raw_scores: bool` (optional) | No | `RerankHit[]` |
| `embed.tokenize` | `tokenize()` | `inputs: string\|string[]` (required); `add_special_tokens: bool` (optional) | No | `u32[][]` |
| `embed.similarity` | `similarity()` | `inputs: [string, string][]` (required) | No | `f32[]` |
| `embed.sparse` | `embed_sparse()` | `inputs: string\|string[]` (required); `truncate: bool` (optional) | No | `SparseToken[][]` |
| `embed.openai` | `openai_embed()` | `body: object` (required; full OpenAI-compat request) | No | `json` |

### `embed.create` param resolution

The dispatcher (`params.rs`) resolves inputs with the following priority:

1. `payload` — full request body object; all other keys are ignored if this is present.
2. `inputs` — string or array of strings.
3. `input` — single string shortcut (used only when `inputs` is absent or empty).

At least one of `input`, `inputs`, or `payload.inputs` must be present; missing inputs
returns a `MissingParam` error.

### `embed.rerank` rate limit

The dispatch layer enforces a hard cap of 100 texts per call. Requests with `texts.len() > 100`
return a `validation_error` with the hint to split into multiple calls of ≤100 texts.

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

Tier-2 shim. `lab tei <action> [--params '<json>']`. The `action` argument defaults to `help`
when omitted.

### MCP (`crates/lab/src/mcp/services/tei.rs`)

Thin bridge re-exporting `ACTIONS` and `dispatch` from `dispatch::tei`. One MCP tool named
`tei`.

### API (`crates/lab/src/api/services/tei.rs`)

`POST /v1/tei` — single route. Reads client from `AppState.clients.tei` (built at startup
from `TEI_URL` and optionally `TEI_API_KEY`). The `help` and `schema` built-ins are routed
through the top-level `dispatch::tei::dispatch` function directly since
`dispatch_with_client` does not handle those actions.

## Config

| Env var | Required | Purpose |
|---------|----------|---------|
| `TEI_URL` | Yes | Base URL of the TEI server |
| `TEI_API_KEY` | No | Bearer token (`Authorization: Bearer <key>`). Omit for unauthenticated local instances. |

## Unimplemented Spec Paths

The following paths from the OpenAPI spec have no SDK method, dispatch action, or surface
wiring:

- `POST /decode`
- `POST /embed_all`
- `GET /metrics`
- `POST /predict`
