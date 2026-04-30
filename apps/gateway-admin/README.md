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

For local binary-served UI work, keep the same-origin `/v1` path and start `lab serve` with web auth disabled for the browser surface only:

```bash
LAB_WEB_UI_AUTH_DISABLED=true \
LAB_MCP_HTTP_TOKEN=your-local-dev-token \
cargo run --bin lab -- serve --host 0.0.0.0 --port 8765
```

That mode keeps the MCP/backend token in place while making `/auth/session` and `/v1/*` immediately usable from the exported `/chat` UI on the same origin. Hosted deployments should leave `LAB_WEB_UI_AUTH_DISABLED` unset so browser OAuth remains active. `LAB_WEB_UI_DISABLE_AUTH` is accepted as a legacy alias.

There is also a repo shortcut for that local ACP UI mode:

```bash
just chat-local
```

Browser-facing bearer mode is intentionally disabled in the current UI. The chat and gateway screens always use the Rust-owned browser session flow plus CSRF headers when talking to `/v1/*`. If you need a local-only backend bypass, use `LAB_WEB_UI_AUTH_DISABLED=true` on the Rust side rather than embedding a public browser token.

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

## Module and Test Tooling Contract

This app is authored as a Next.js bundler-style ESM project. The canonical module
model is:

- `package.json` sets `"type": "module"`
- `tsconfig.json` uses `"module": "esnext"` and `"moduleResolution": "bundler"`
- absolute app imports use the `@/*` alias from `tsconfig.json`
- local relative imports may omit extensions when they are consumed by Next or
  the `tsx` test runner

The default verification path follows that same model:

```bash
pnpm test
```

`pnpm test` runs the Node-compatible unit suite through `tsx`, which honors the
same TypeScript and ESM assumptions used by the app. Browser-only checks remain
under `pnpm run test:browser`; they are intentionally separate from the default
unit gate.

Full `tsc --noEmit -p tsconfig.json` is not yet the default gate because older
UI surfaces still have unrelated strict-type debt. Do not add a package-level
`typecheck` script until that full command is green; otherwise contributors get
a verification command whose failures are unrelated to the module contract.

`lib/tooling-contract.test.ts` guards this contract so future script or
`tsconfig.json` changes do not silently split authoring, typecheck, and test
resolution semantics.

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

ACP event handling details:
- reconnects resume from the last seen sequence number for the active session
- the client keeps a bounded in-memory history instead of growing without limit
- duplicate or out-of-order events are ignored

## ACP stream lifecycle contract

### Ordering guarantees

Events within a session carry a monotonically increasing `seq` number assigned by a
mutex-guarded counter in the server registry. A single fanout task writes to all
subscribers, so ordering is preserved end-to-end. Events from different sessions have
independent counters and are not ordered relative to each other.

### Replay and resume semantics

The SSE endpoint accepts an optional `?since=<seq>` query parameter (unsigned integer,
defaults to `0`). On connect the server:

1. Registers the subscriber channel **before** querying the backlog, eliminating the
   subscribe-first race condition where live events could arrive between the backlog
   query and channel registration.
2. Loads events with `seq > since` from the SQLite store (up to the 10 000-event
   backfill cap).
3. Falls back to the in-memory transcript when SQLite is unavailable.
4. Chains the backlog stream with the live channel stream. At the junction, live events
   whose `seq` is at or below the last backlog event are discarded to prevent duplicates.

Clients should persist the highest `seq` seen and pass it as `since` on reconnect to
receive only events missed during the disconnection window.

### Retention and windowing

SQLite is the authoritative event store. There is no server-side expiry policy: events
persist for the lifetime of the session unless explicitly deleted.

When SQLite is unavailable the server falls back to an in-memory transcript capped at
500 events per session (FIFO eviction). Under the fallback path a reconnect with a
`since` value that predates the oldest in-memory event will receive a silent partial
backlog — no gap marker is emitted. Clients that need reliable full replay should
operate against a deployment where SQLite persistence is enabled.

Regardless of persistence mode, `subscribe()` returns at most 10 000 events per
reconnect (the `BACKFILL_CAP`).

### Backpressure and subscriber eviction

Each subscriber receives events through a per-subscriber bounded channel (capacity 64).
If a subscriber cannot drain the channel fast enough and the buffer fills, the server:

1. Drops the slow subscriber immediately.
2. Emits a synthetic `ProviderInfo` event with `type: "subscriber_backpressure"` into
   the remaining stream and persists it so it is visible in replays.
3. Logs a `WARN` with `action = "fanout.backpressure"` and the `after_seq` at which
   the gap begins.

A client that is evicted will stop receiving events silently. Clients should treat a
dropped SSE connection as a signal to reconnect with their last-seen `seq`.

### `_meta` field relay

Tool-call events may carry an optional `_meta` object injected by the originating agent.
Lab relays this field transparently — it does not validate, transform, or construct `_meta`
content.

- **Live delivery:** `_meta` is forwarded as-is to SSE subscribers.
- **Replay:** `_meta` survives the SQLite round-trip and is present in replayed events.
- **Redaction on replay:** The standard field-name redaction policy applies recursively
  to the entire persisted event payload — not only to `_meta` subfields. Any JSON key
  matching a sensitive name is replaced with `"[REDACTED]"` before storage. Sensitive
  key names include: `token`, `access_token`, `id_token`, `refresh_token`, `apikey`,
  `api_key`, `password`, `passwd`, `secret`, `client_secret`, `authorization`,
  `bearer`, `session`, `session_id`, `cookie`, `code`, `cwd`, `terminal_id`, and any
  key ending with `_token`, `_secret`, `_password`, or `_key`. Agents must not use
  these names as correlation keys anywhere in the event payload.
- **Absent vs null:** When the originating agent did not inject `_meta`, the key is
  absent from the event entirely (not emitted as `null`).
