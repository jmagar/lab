# HF TEI API Coverage

**Last updated:** 2026-04-12
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

The source spec is the contract. This document is the implementation planning aid.

## Text Embeddings Inference

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| POST | /decode | decode | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /embed | embed | ✅ | ✅ | ✅ | ✅ |
| POST | /embed_all | embed_all | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /embed_sparse | embed_sparse | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /health | health | ✅ | ✅ | ✅ | ✅ |
| GET | /info | get_model_info | ✅ | ✅ | ✅ | ✅ |
| GET | /metrics | metrics | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /predict | predict | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /rerank | rerank | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /similarity | similarity | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /tokenize | tokenize | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /v1/embeddings | openai_embed | ⬜ | ⬜ | ⬜ | ⬜ |
