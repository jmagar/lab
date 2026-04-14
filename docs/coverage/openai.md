# OpenAI API Coverage

**Last updated:** 2026-04-13
**OpenAPI spec:** docs/api-specs/openai.openapi.yaml
**OpenAPI version:** 3.0.0
**API version:** 2.3.0
**Paths:** 148
**Servers:** https://api.openai.com/v1
**Security schemes:** ApiKeyAuth (Bearer token via `Authorization: Bearer <key>`)
**Compatible with:** Any OpenAI-compatible server (Ollama, vLLM, LiteLLM, etc.) via `OPENAI_URL` override.

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented across all surfaces (SDK client, MCP, CLI, HTTP API) |
| ⬜ | Not implemented; row is spec inventory only |
| - | Not applicable / not represented in the spec |

The source spec is the contract. This document is the implementation planning aid.

## Implemented Actions

These actions are fully wired: `lab-apis` client method → dispatch layer → MCP + CLI + HTTP API.

| Action | HTTP Method | Endpoint | SDK Method | MCP | CLI | API |
|--------|-------------|----------|------------|-----|-----|-----|
| `server.health` | GET | /v1/models | `health()` | ✅ | ✅ | ✅ |
| `model.list` | GET | /v1/models | `list_models()` | ✅ | ✅ | ✅ |
| `chat.complete` | POST | /v1/chat/completions | `chat_completion()` | ✅ | ✅ | ✅ |
| `embed.create` | POST | /v1/embeddings | `create_embeddings()` | ✅ | ✅ | ✅ |

### `server.health`
- **Params:** none
- **Returns:** `{ "ok": true }`
- **Notes:** Pings `GET /v1/models` as a cheap health probe. Succeeds if the server is
  reachable and the API key has at least read scope.

### `model.list`
- **Params:** none
- **Returns:** `ModelsResponse` — `{ "object": "list", "data": [{ "id", "object", "created", "owned_by" }, ...] }`

### `chat.complete`
- **Params:**
  - `model` (string, required) — model identifier (e.g. `gpt-4o-mini`, `llama3`)
  - `messages` (json array, required) — `[{"role":"user|system|assistant|tool","content":"..."}]`. Also accepted as a JSON-encoded string.
  - `temperature` (number, optional) — sampling temperature (0.0–2.0)
  - `max_tokens` (integer, optional) — maximum tokens to generate
- **Returns:** `ChatCompletionResponse` — `{ "id", "object", "created", "model", "choices": [{ "index", "message": {"role","content"}, "finish_reason" }], "usage": {"prompt_tokens","completion_tokens","total_tokens"} }`
- **Notes:** `stream` is never set (non-streaming only).

### `embed.create`
- **Params:**
  - `model` (string, required) — embedding model (e.g. `text-embedding-3-small`)
  - `input` (string, optional) — single input string shortcut
  - `inputs` (json array, optional) — array of input strings (batch mode). Use `input` OR `inputs`, not both.
  - `dimensions` (integer, optional) — output embedding dimension
- **Returns:** `EmbeddingsResponse` — `{ "object", "data": [{ "index", "embedding": [f32,...], "object" }], "model", "usage" }`

## Config / Auth

| Env Var | Required | Description |
|---------|----------|-------------|
| `OPENAI_API_KEY` | yes | Bearer token sent as `Authorization: Bearer <key>` |
| `OPENAI_URL` | no | Base URL (default: `https://api.openai.com`) |

Setting `OPENAI_URL` to a local server (e.g. `http://localhost:11434`) enables OpenAI-compatible servers.

## CLI Usage

```bash
lab openai help
lab openai model.list
lab openai chat.complete --params '{"model":"gpt-4o-mini","messages":[{"role":"user","content":"Hello"}]}'
lab openai embed.create --params '{"model":"text-embedding-3-small","input":"Hello world"}'
lab openai server.health
```

The CLI is a flat action+params shim (`OpenaiArgs { action, params }`). There are no typed subcommands.

## Not Yet Implemented

All other OpenAI API paths (148 total in spec). The following sections are spec inventory only — none have SDK client methods, dispatch actions, or surface wiring.

### Assistants

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /assistants | listAssistants | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /assistants | createAssistant | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /assistants/{assistant_id} | deleteAssistant | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /assistants/{assistant_id} | getAssistant | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /assistants/{assistant_id} | modifyAssistant | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/admin_api_keys | admin-api-keys-list | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /organization/admin_api_keys | admin-api-keys-create | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /organization/admin_api_keys/{key_id} | admin-api-keys-delete | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/admin_api_keys/{key_id} | admin-api-keys-get | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /threads | createThread | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /threads/runs | createThreadAndRun | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /threads/{thread_id} | deleteThread | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /threads/{thread_id} | getThread | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /threads/{thread_id} | modifyThread | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /threads/{thread_id}/messages | listMessages | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /threads/{thread_id}/messages | createMessage | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /threads/{thread_id}/messages/{message_id} | deleteMessage | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /threads/{thread_id}/messages/{message_id} | getMessage | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /threads/{thread_id}/messages/{message_id} | modifyMessage | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /threads/{thread_id}/runs | listRuns | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /threads/{thread_id}/runs | createRun | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /threads/{thread_id}/runs/{run_id} | getRun | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /threads/{thread_id}/runs/{run_id} | modifyRun | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /threads/{thread_id}/runs/{run_id}/cancel | cancelRun | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /threads/{thread_id}/runs/{run_id}/steps | listRunSteps | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /threads/{thread_id}/runs/{run_id}/steps/{step_id} | getRunStep | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /threads/{thread_id}/runs/{run_id}/submit_tool_outputs | submitToolOutputsToRun | ⬜ | ⬜ | ⬜ | ⬜ |

### Audio

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| POST | /audio/speech | createSpeech | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /audio/transcriptions | createTranscription | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /audio/translations | createTranslation | ⬜ | ⬜ | ⬜ | ⬜ |

### Chat (stored completions — distinct from `chat.complete`)

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /chat/completions | listChatCompletions | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /chat/completions/{completion_id} | deleteChatCompletion | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /chat/completions/{completion_id} | getChatCompletion | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /chat/completions/{completion_id} | updateChatCompletion | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /chat/completions/{completion_id}/messages | getChatCompletionMessages | ⬜ | ⬜ | ⬜ | ⬜ |

### Completions (legacy)

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| POST | /completions | createCompletion | ⬜ | ⬜ | ⬜ | ⬜ |

### Evals

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /evals | listEvals | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /evals | createEval | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /evals/{eval_id} | deleteEval | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /evals/{eval_id} | getEval | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /evals/{eval_id} | updateEval | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /evals/{eval_id}/runs | getEvalRuns | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /evals/{eval_id}/runs | createEvalRun | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /evals/{eval_id}/runs/{run_id} | deleteEvalRun | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /evals/{eval_id}/runs/{run_id} | getEvalRun | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /evals/{eval_id}/runs/{run_id} | cancelEvalRun | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /evals/{eval_id}/runs/{run_id}/output_items | getEvalRunOutputItems | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /evals/{eval_id}/runs/{run_id}/output_items/{output_item_id} | getEvalRunOutputItem | ⬜ | ⬜ | ⬜ | ⬜ |

### Fine-tuning

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions | listFineTuningCheckpointPermissions | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions | createFineTuningCheckpointPermission | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /fine_tuning/checkpoints/{fine_tuned_model_checkpoint}/permissions/{permission_id} | deleteFineTuningCheckpointPermission | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /fine_tuning/jobs | listPaginatedFineTuningJobs | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /fine_tuning/jobs | createFineTuningJob | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /fine_tuning/jobs/{fine_tuning_job_id} | retrieveFineTuningJob | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /fine_tuning/jobs/{fine_tuning_job_id}/cancel | cancelFineTuningJob | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /fine_tuning/jobs/{fine_tuning_job_id}/checkpoints | listFineTuningJobCheckpoints | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /fine_tuning/jobs/{fine_tuning_job_id}/events | listFineTuningEvents | ⬜ | ⬜ | ⬜ | ⬜ |

### Batch

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /batches | listBatches | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /batches | createBatch | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /batches/{batch_id} | retrieveBatch | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /batches/{batch_id}/cancel | cancelBatch | ⬜ | ⬜ | ⬜ | ⬜ |

### Files

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /files | listFiles | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /files | createFile | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /files/{file_id} | deleteFile | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /files/{file_id} | retrieveFile | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /files/{file_id}/content | downloadFile | ⬜ | ⬜ | ⬜ | ⬜ |

### Uploads

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| POST | /uploads | createUpload | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /uploads/{upload_id}/cancel | cancelUpload | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /uploads/{upload_id}/complete | completeUpload | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /uploads/{upload_id}/parts | addUploadPart | ⬜ | ⬜ | ⬜ | ⬜ |

### Images

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| POST | /images/edits | createImageEdit | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /images/generations | createImage | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /images/variations | createImageVariation | ⬜ | ⬜ | ⬜ | ⬜ |

### Models (management — delete/retrieve by id)

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| DELETE | /models/{model} | deleteModel | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /models/{model} | retrieveModel | ⬜ | ⬜ | ⬜ | ⬜ |

### Moderations

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| POST | /moderations | createModeration | ⬜ | ⬜ | ⬜ | ⬜ |

### Audit Logs

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /organization/audit_logs | list-audit-logs | ⬜ | ⬜ | ⬜ | ⬜ |

### Certificates

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /organization/certificates | listOrganizationCertificates | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /organization/certificates | uploadCertificate | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /organization/certificates/activate | activateOrganizationCertificates | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /organization/certificates/deactivate | deactivateOrganizationCertificates | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /organization/certificates/{certificate_id} | deleteCertificate | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/certificates/{certificate_id} | getCertificate | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /organization/certificates/{certificate_id} | modifyCertificate | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/projects/{project_id}/certificates | listProjectCertificates | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /organization/projects/{project_id}/certificates/activate | activateProjectCertificates | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /organization/projects/{project_id}/certificates/deactivate | deactivateProjectCertificates | ⬜ | ⬜ | ⬜ | ⬜ |

### Usage

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /organization/costs | usage-costs | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/usage/audio_speeches | usage-audio-speeches | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/usage/audio_transcriptions | usage-audio-transcriptions | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/usage/code_interpreter_sessions | usage-code-interpreter-sessions | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/usage/completions | usage-completions | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/usage/embeddings | usage-embeddings | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/usage/images | usage-images | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/usage/moderations | usage-moderations | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/usage/vector_stores | usage-vector-stores | ⬜ | ⬜ | ⬜ | ⬜ |

### Invites

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /organization/invites | list-invites | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /organization/invites | inviteUser | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /organization/invites/{invite_id} | delete-invite | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/invites/{invite_id} | retrieve-invite | ⬜ | ⬜ | ⬜ | ⬜ |

### Projects

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /organization/projects | list-projects | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /organization/projects | create-project | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/projects/{project_id} | retrieve-project | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /organization/projects/{project_id} | modify-project | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/projects/{project_id}/api_keys | list-project-api-keys | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /organization/projects/{project_id}/api_keys/{key_id} | delete-project-api-key | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/projects/{project_id}/api_keys/{key_id} | retrieve-project-api-key | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /organization/projects/{project_id}/archive | archive-project | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/projects/{project_id}/rate_limits | list-project-rate-limits | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /organization/projects/{project_id}/rate_limits/{rate_limit_id} | update-project-rate-limits | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/projects/{project_id}/service_accounts | list-project-service-accounts | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /organization/projects/{project_id}/service_accounts | create-project-service-account | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /organization/projects/{project_id}/service_accounts/{service_account_id} | delete-project-service-account | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/projects/{project_id}/service_accounts/{service_account_id} | retrieve-project-service-account | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/projects/{project_id}/users | list-project-users | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /organization/projects/{project_id}/users | create-project-user | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /organization/projects/{project_id}/users/{user_id} | delete-project-user | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/projects/{project_id}/users/{user_id} | retrieve-project-user | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /organization/projects/{project_id}/users/{user_id} | modify-project-user | ⬜ | ⬜ | ⬜ | ⬜ |

### Users

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /organization/users | list-users | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /organization/users/{user_id} | delete-user | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /organization/users/{user_id} | retrieve-user | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /organization/users/{user_id} | modify-user | ⬜ | ⬜ | ⬜ | ⬜ |

### Realtime

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| POST | /realtime/sessions | create-realtime-session | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /realtime/transcription_sessions | create-realtime-transcription-session | ⬜ | ⬜ | ⬜ | ⬜ |

### Responses

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| POST | /responses | createResponse | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /responses/{response_id} | deleteResponse | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /responses/{response_id} | getResponse | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /responses/{response_id}/input_items | listInputItems | ⬜ | ⬜ | ⬜ | ⬜ |

### Vector stores

| Method | Endpoint | SDK Method | Impl | MCP | CLI | API |
|--------|----------|------------|------|-----|-----|-----|
| GET | /vector_stores | listVectorStores | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /vector_stores | createVectorStore | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /vector_stores/{vector_store_id} | deleteVectorStore | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /vector_stores/{vector_store_id} | getVectorStore | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /vector_stores/{vector_store_id} | modifyVectorStore | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /vector_stores/{vector_store_id}/file_batches | createVectorStoreFileBatch | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /vector_stores/{vector_store_id}/file_batches/{batch_id} | getVectorStoreFileBatch | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /vector_stores/{vector_store_id}/file_batches/{batch_id}/cancel | cancelVectorStoreFileBatch | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /vector_stores/{vector_store_id}/file_batches/{batch_id}/files | listFilesInVectorStoreBatch | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /vector_stores/{vector_store_id}/files | listVectorStoreFiles | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /vector_stores/{vector_store_id}/files | createVectorStoreFile | ⬜ | ⬜ | ⬜ | ⬜ |
| DELETE | /vector_stores/{vector_store_id}/files/{file_id} | deleteVectorStoreFile | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /vector_stores/{vector_store_id}/files/{file_id} | getVectorStoreFile | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /vector_stores/{vector_store_id}/files/{file_id} | updateVectorStoreFileAttributes | ⬜ | ⬜ | ⬜ | ⬜ |
| GET | /vector_stores/{vector_store_id}/files/{file_id}/content | retrieveVectorStoreFileContent | ⬜ | ⬜ | ⬜ | ⬜ |
| POST | /vector_stores/{vector_store_id}/search | searchVectorStore | ⬜ | ⬜ | ⬜ | ⬜ |
