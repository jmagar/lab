# Gateway Admin UI

Static-exportable Next.js admin UI for the `gateway` surface, vendored into the `lab` repo at `apps/gateway-admin`.

The app is designed to be served as static assets while talking directly to the Rust gateway backend over HTTP. It can still be run in standalone frontend mode during development.

## Current State

- App framework: Next.js 16 + React 19
- Package manager: `pnpm` (lockfile included)
- Data mode: browser client over the Rust `/v1/gateway` endpoint, with same-origin browser session auth for hosted deployments and optional mock data for local UI work
- Security rule: browser-facing flows must use backend-supported redacted payloads for any response that can contain secrets. The Setup page requests redacted extract scan results and never receives raw extracted secret values.

## Local Usage

From this directory:

```bash
pnpm install
pnpm dev
```

The dev server binds on `0.0.0.0` so the UI is reachable from other devices on the local network during development.

The app defaults `NEXT_PUBLIC_API_URL` to `/v1`, which is the expected same-origin path once `lab serve` hosts both API and UI. Override it when pointing the UI at a different backend origin.

```bash
NEXT_PUBLIC_API_URL=http://127.0.0.1:8765/v1 pnpm dev
```

In hosted mode, the UI expects Rust-owned browser session auth:

- `GET /auth/session` boots the browser auth state
- `GET /auth/login` starts the Rust-owned login flow
- `POST /auth/logout` clears the browser session
- `/v1/*` uses same-origin requests with `credentials: 'include'`

If the backend is protected by a static bearer token, set `NEXT_PUBLIC_API_TOKEN`. Bearer mode activates automatically — no additional flag required. This is suitable for local development, smoke testing, and browser automation against OAuth-protected deployments. The token is embedded into the browser bundle, so it should not be used for production hosted deployments where Rust-owned browser session auth is preferred.

```bash
NEXT_PUBLIC_API_URL=http://127.0.0.1:8765/v1 \
NEXT_PUBLIC_API_TOKEN=your-local-dev-token \
pnpm dev
```

When the frontend and Rust backend run on different origins during local development, the backend must allow the frontend origin through CORS:

```bash
LAB_MCP_HTTP_TOKEN=your-local-dev-token \
LAB_CORS_ORIGINS=http://127.0.0.1:3101 \
cargo run --bin lab -- serve --host 0.0.0.0 --port 8765
```

```bash
NEXT_PUBLIC_API_URL=http://127.0.0.1:8765/v1 \
NEXT_PUBLIC_API_TOKEN=your-local-dev-token \
pnpm dev --hostname 127.0.0.1 --port 3101
```

## Marketplace Editing Workflow

Marketplace package files are edited through the shared CodeMirror surface in the UI.

- `Save` writes the current file into an app-managed workspace mirror
- `Deploy` is explicit and syncs the saved workspace into the local Claude Code target
- save and deploy are intentionally separate so draft iteration does not immediately affect the installed Claude Code files
- plugin details use the dedicated route `/marketplace/plugin?id=<pluginId>` rather than an in-page modal
- the plugin `Files` tab supports deploy preview before the explicit deploy step

## Static Export

Build the export artifact:

```bash
pnpm build
```

This writes the static site to `out/`.

Preview the exported assets locally:

```bash
pnpm start
```

## Notes

- The imported UI code was originally developed as its own repository and is now tracked as normal source under this repo.
- Nested git metadata was removed on import so `apps/gateway-admin` behaves like a standard in-repo app directory.

## ACP bridge

The chat view can run against a local ACP bridge instead of mock transcript data.

Environment variables:
- `ACP_CODEX_COMMAND`: optional override for the ACP provider executable.
- `ACP_CODEX_ARGS`: optional space-delimited args for `ACP_CODEX_COMMAND`.
- `ACP_SESSION_CWD`: optional default working directory for newly created ACP sessions.

By default, the bridge launches `codex-acp` from the Rust backend via `npx @zed-industries/codex-acp` and exposes:
- `GET /v1/acp/provider`
- `GET /v1/acp/sessions`
- `POST /v1/acp/sessions`
- `POST /v1/acp/sessions/{sessionId}/prompt`
- `POST /v1/acp/sessions/{sessionId}/cancel`
- `GET /v1/acp/sessions/{sessionId}/events`

The static browser app consumes live ACP session updates over SSE from the Rust API and renders them into the transcript lane plus the `Reasoning & Activity` timeline.
