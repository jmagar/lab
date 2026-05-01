# OpenACP Service Coverage

Updated: 2026-05-01

`openacp` is an upstream OpenACP daemon integration. It is separate from Lab's
internal `acp` service.

OpenACP actions are intentionally not Lab-destructive and do not require Lab
confirmation gates. This includes prompt/session, bypass, permission, config,
topic, tunnel, notify, and restart operations.

## Environment

- `OPENACP_URL` - base URL, for example `http://127.0.0.1:21420`
- `OPENACP_TOKEN` - bearer token or scoped JWT
- Named instances use `OPENACP_<LABEL>_URL` and `OPENACP_<LABEL>_TOKEN`

Lab does not auto-read OpenACP `api-secret` files.

## Coverage Matrix

| Lab action | Upstream path | Status | Lab destructive |
|---|---|---:|---:|
| `system.health` | `GET /api/health` | implemented | false |
| `system.version` | `GET /api/version` | implemented | false |
| `system.restart` | `POST /api/restart` | implemented | false |
| `adapters.list` | `GET /api/adapters` | implemented | false |
| `sessions.list` | `GET /api/sessions` | implemented | false |
| `sessions.get` | `GET /api/sessions/:id` | implemented | false |
| `sessions.create` | `POST /api/sessions` | implemented | false |
| `sessions.prompt` | `POST /api/sessions/:id/prompt` | implemented | false |
| `sessions.cancel` | `DELETE /api/sessions/:id` | implemented | false |
| `sessions.bypass.set` | `PATCH /api/sessions/:id/bypass` | implemented | false |
| `sessions.permission.resolve` | `POST /api/sessions/:id/permission` | implemented | false |
| `sessions.archive` | `POST /api/sessions/:id/archive` | implemented | false |
| `sessions.adopt` | `POST /api/sessions/adopt` | implemented | false |
| `agents.list` | `GET /api/agents` | implemented | false |
| `config.get` | `GET /api/config` | implemented | false |
| `config.editable` | `GET /api/config/editable` | implemented | false |
| `config.patch` | `PATCH /api/config` | implemented | false |
| `topics.list` | `GET /api/topics` | implemented | false |
| `topics.delete` | `DELETE /api/topics/:sessionId` | implemented | false |
| `topics.cleanup` | `POST /api/topics/cleanup` | implemented | false |
| `tunnel.status` | `GET /api/tunnel` | implemented | false |
| `tunnel.list` | `GET /api/tunnel/list` | implemented | false |
| `tunnel.create` | `POST /api/tunnel` | implemented | false |
| `tunnel.delete` | `DELETE /api/tunnel/:port` | implemented | false |
| `tunnel.delete_all` | `DELETE /api/tunnel` | implemented | false |
| `notify.send` | `POST /api/notify` | implemented | false |
| auth token management | `/api/v1/auth/*` | deferred | false |
| SSE event streams | `/api/events`, `/api/v1/sse/*` | deferred | false |

Implemented actions: 26. Deferred endpoint groups: 2.

## Live Test Evidence

Pending. No local OpenACP daemon was confirmed during implementation.

Planned smoke commands:

```bash
OPENACP_URL=http://127.0.0.1:21420 lab openacp system.health --json
mcporter call openacp '{"action":"system.health","params":{}}'
curl -sS -X POST http://127.0.0.1:<lab-port>/v1/openacp \
  -H 'Content-Type: application/json' \
  -d '{"action":"system.health","params":{}}'
```

Observability evidence is also pending live daemon availability. Expected
evidence: one successful dispatch and one failing dispatch with no bearer
token, prompt body, raw config dump, workspace path, or session payload in logs.
