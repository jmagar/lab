# Gateway Admin UI

Static-exportable Next.js admin UI for the `gateway` surface, vendored into the `lab` repo at `apps/gateway-admin`.

The app is designed to be served as static assets while talking directly to the Rust gateway backend over HTTP. It can still be run in standalone frontend mode during development.

## Current State

- App framework: Next.js 16 + React 19
- Package manager: `pnpm` (lockfile included)
- Data mode: browser client over the Rust `/v1/gateway` endpoint, with optional mock data for local UI work

## Local Usage

From this directory:

```bash
pnpm install
pnpm dev
```

The app defaults `NEXT_PUBLIC_API_URL` to `/v1`, which is the expected same-origin path once `lab serve` hosts both API and UI. Override it when pointing the UI at a different backend origin.

```bash
NEXT_PUBLIC_API_URL=http://127.0.0.1:8765/v1 pnpm dev
```

If the backend is protected by a static bearer token and you need a standalone browser build to talk to it directly, set `NEXT_PUBLIC_API_TOKEN` as well. This is suitable for local development and smoke testing only because the token is embedded into the browser bundle.

```bash
NEXT_PUBLIC_API_URL=http://127.0.0.1:8765/v1 \
NEXT_PUBLIC_API_TOKEN=your-local-dev-token \
pnpm dev
```

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
